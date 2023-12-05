use anyhow::{bail, Result};
use graphql_client::GraphQLQuery;

use crate::GitHubRepo;

const API_URL: &str = "https://api.github.com/graphql";

pub struct GitHub {
    client: reqwest::Client,
}

type Cursor = String;

trait ChunkedQuery: GraphQLQuery {
    type Item;

    fn change_after(&self, v: Self::Variables, after: Option<String>) -> Self::Variables;
    fn set_batch(&self, batch: i64, v: Self::Variables) -> Self::Variables;

    fn process(&self, d: Self::ResponseData) -> Result<(Vec<Self::Item>, Option<Cursor>)>;
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

impl ChunkedQuery for PullsQuery {
    type Item = PullRequest;

    fn change_after(&self, v: Self::Variables, after: Option<String>) -> Self::Variables {
        Self::Variables { after, ..v }
    }
    fn set_batch(&self, batch: i64, v: Self::Variables) -> Self::Variables {
        Self::Variables { batch, ..v }
    }

    fn process(&self, d: Self::ResponseData) -> Result<(Vec<Self::Item>, Option<Cursor>)> {
        debug!("rate limits: {:?}", d.rate_limit);
        let prs = match d.repository {
            Some(r) => r.pull_requests,
            None => bail!("query returned no repo"),
        };
        // deliberately ignore all nulls. no idea why the schema doesn't make
        // all of these links mandatory, having them nullable makes no sense.
        let infos: Vec<PullRequest> = prs
            .edges
            .unwrap_or_default()
            .into_iter()
            .filter_map(|e| e?.node)
            .map(|n| PullRequest {
                id: n.id,
                title: n.title,
                is_open: !n.closed,
                is_merged: n.merged,
                body: n.body_html,
                last_update: n.updated_at,
                url: n.url,
                base_ref: n.base_ref_name,
                merge_commit: n.merge_commit.map(|c| c.oid),
                landed_in: BTreeSet::default(),
            })
            .collect();
        let cursor = match (self.since, infos.last()) {
            (Some(since), Some(last)) if last.last_update < since => None,
            _ => {
                if prs.page_info.has_next_page {
                    prs.page_info.end_cursor
                } else {
                    None
                }
            }
        };
        Ok((infos, cursor))
    }
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
