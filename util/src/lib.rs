use std::time::{Duration, Instant};
use std::process::{Child, Command};

use camino::{Utf8Path, Utf8PathBuf};
use sqlx::{Connection, PgConnection, PgPool};

pub struct DatabaseContext {
    tmp_dir: tempfile::TempDir,
    postgres: Child,
}

impl Drop for DatabaseContext {
    fn drop(&mut self) {
        tokio::block_on;
        self.postgres.kill().unwrap();
        self.postgres.wait().unwrap();
    }
}

impl DatabaseContext {
    // Will not be used as port, but as part of socket filename.
    // See `listen_addresses` below.
    const PORT: &str = "1";

    fn sockets_dir(path: &Utf8Path) -> Utf8PathBuf {
        path.join("sockets")
    }

    async fn connection(&self) -> Result<PgConnection, sqlx::Error> {
        let url = self.db_url();
        sqlx::PgConnection::connect(&url).await
    }

    async fn pool(&self) -> Result<PgPool, sqlx::Error> {
        let url = self.db_url();
        sqlx::PgPool::connect(&url).await
    }

    pub async fn init() -> Self {
        let tmp_dir = tempfile::tempdir().unwrap();
        let sockets_dir = Self::sockets_dir(tmp_dir.path().try_into().unwrap());
        let data_dir = tmp_dir.path().join("data");
        std::fs::create_dir(&sockets_dir).unwrap();

        assert!(Command::new("initdb")
            .arg(&data_dir)
            .status()
            // .await
            .unwrap()
            .success());

        let postgres = Command::new("postgres")
            .arg("-D")
            .arg(data_dir)
            .arg("-p")
            .arg(Self::PORT)
            .arg("-c")
            .arg(format!("unix_socket_directories={sockets_dir}"))
            .arg("-c")
            .arg("listen_addresses=")
            .spawn()
            .unwrap();

        let socket_path = sockets_dir.join(format!(".s.PGSQL.{}", Self::PORT));

        let started = Instant::now();

        while !socket_path.exists() {
            assert!(
                started.elapsed() < Duration::from_secs(5),
                "db should start within 5 seconds"
            );
            std::thread::sleep(Duration::from_millis(10));
        }

        let this = Self { tmp_dir, postgres };
        sqlx::migrate!("../store/migrations")
            .run(&this.pool().await.unwrap())
            .await;

        this
    }

    pub fn db_url(&self) -> String {
        let dbname = "postgres"; // TODO

        format!(
            "postgresql:///{dbname}?host={}&port={}",
            Self::sockets_dir(self.tmp_dir.path().try_into().unwrap()),
            Self::PORT,
        )
    }
}
