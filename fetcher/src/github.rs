use graphql_client::GraphQLQuery;

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

impl Github {
    pub fn new(api_token: &str, owner: &str, repo: &str, label: &str) -> Result<Self> {
        use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};

        let headers = match HeaderValue::from_str(&format!("Bearer {api_token}")) {
            Ok(h) => [(AUTHORIZATION, h)].into_iter().collect::<HeaderMap>(),
            Err(e) => bail!("invalid API token: {}", e),
        };
        let client = reqwest::blocking::Client::builder()
            .user_agent(format!(
                "{}/{}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            ))
            .default_headers(headers)
            .build()?;
        Ok(Github {
            client,
            owner: owner.to_string(),
            repo: repo.to_string(),
            label: label.to_string(),
        })
    }
}
