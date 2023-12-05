use anyhow::{bail, Result};
use graphql_client::GraphQLQuery;

use crate::GitHubRepo;

const API_URL: &str = "https://api.github.com/graphql";

pub struct GitHub {
    client: reqwest::Client,
}

type DateTime = chrono::DateTime<chrono::Utc>;
type URI = String;
type HTML = String;
type GitObjectID = String;

#[derive(Debug, GraphQLQuery)]
#[graphql(
    schema_path = "../vendor/github.com/schema.docs.graphql",
    query_path = "src/queries/pulls.graphql",
    response_derives = "Debug",
    variables_derives = "Clone,Debug"
)]
pub struct PullsQuery {
    since: Option<DateTime>,
}

#[derive(Debug, GraphQLQuery)]
#[graphql(
    schema_path = "../vendor/github.com/schema.docs.graphql",
    query_path = "src/queries/branch_contains.graphql",
    response_derives = "Debug",
    variables_derives = "Clone,Debug"
)]
pub struct BranchContainsQuery {
    since: Option<DateTime>,
}

impl GitHub {
    pub fn new(api_token: &str) -> Result<Self> {
        use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};

        let headers = match HeaderValue::from_str(&format!("Bearer {api_token}")) {
            Ok(h) => [(AUTHORIZATION, h)].into_iter().collect::<HeaderMap>(),
            Err(e) => bail!("invalid API token: {}", e),
        };
        let client = reqwest::Client::builder()
            .user_agent(format!(
                "{}/{}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            ))
            .default_headers(headers)
            .build()?;
        Ok(Self { client })
    }

    pub fn get_pulls(&self, repo: GitHubRepo) {}
}
