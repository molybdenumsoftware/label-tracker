use std::process::Child;

struct DatabaseContext {
    tmp_dir: tempfile::TempDir,
    postgres: Child,
}

impl Drop for DatabaseContext {
    fn drop(&mut self) {
        self.postgres.kill().unwrap();
        self.postgres.wait().unwrap();
    }
}

impl DatabaseContext {
    // Note: postgres isn't actually going to listen on this port (see the empty
    // listen_addresses down below), this just determines the name of the socket it listens to.
    const PORT: &str = "1";

    fn sockets_dir(path: &Utf8Path) -> Utf8PathBuf {
        path.join("sockets")
    }

    fn init() -> Self {
        let tmp_dir = tempfile::tempdir().unwrap();
        let sockets_dir = Self::sockets_dir(tmp_dir.path().try_into().unwrap());
        let data_dir = tmp_dir.path().join("data");
        fs::create_dir(&sockets_dir).unwrap();

        assert!(Command::new("initdb")
            .arg(&data_dir)
            .status()
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

        let n = Instant::now();

        while !socket_path.exists() {
            assert!(
                n.elapsed() < Duration::from_secs(5),
                "db should start within 5 seconds"
            );
            thread::sleep(Duration::from_millis(10));
        }

        Self { tmp_dir, postgres }
    }

    fn db_url(&self) -> String {
        let dbname = "postgres"; // TODO

        format!(
            "postgresql:///{dbname}?host={}&port={}",
            Self::sockets_dir(self.tmp_dir.path().try_into().unwrap()),
            Self::PORT,
        )
    }
}
