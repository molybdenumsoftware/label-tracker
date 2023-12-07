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
    type Item = store::Pr;

    fn change_after(&self, v: Self::Variables, after: Option<String>) -> Self::Variables {
        Self::Variables { after, ..v }
    }
    fn set_batch(&self, batch: i64, v: Self::Variables) -> Self::Variables {
        Self::Variables { batch, ..v }
    }

    fn process(&self, d: Self::ResponseData) -> Result<(Vec<Self::Item>, Option<Cursor>)> {
        log::debug!("rate limits: {:?}", d.rate_limit);
        let prs = match d.repository {
            Some(r) => r.pull_requests,
            None => bail!("query returned no repo"),
        };
        // deliberately ignore all nulls. no idea why the schema doesn't make
        // all of these links mandatory, having them nullable makes no sense.
        let infos: Vec<store::Pr> = prs
            .edges
            .unwrap_or_default()
            .into_iter()
            .filter_map(|e| e?.node)
            .filter_map(|n| {
                Some(store::Pr {
                    number: store::PrNumber(
                        n.number
                            .try_into()
                            .expect("pr should be less than i32::MAX"),
                    ),
                    commit: store::GitCommit(n.merge_commit?.oid),
                })
            })
            .collect();
        // TODO stop processing old prs
        // let cursor = match (self.since, infos.last()) {
        //     (Some(since), Some(last)) if last.last_update < since => None,
        //     _ => {
        //         if prs.page_info.has_next_page {
        //             prs.page_info.end_cursor
        //         } else {
        //             None
        //         }
        //     }
        // };
        Ok((infos, prs.page_info.end_cursor))
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
    async fn query_raw<Q>(
        &self,
        q: &Q,
        mut vars: <Q as GraphQLQuery>::Variables,
    ) -> Result<Vec<Q::Item>>
    where
        Q: ChunkedQuery + std::fmt::Debug,
        Q::Variables: Clone + std::fmt::Debug,
    {
        let mut result = vec![];
        let max_batch = 100;
        let mut batch = max_batch;

        loop {
            vars = q.set_batch(batch, vars);

            log::debug!("running query {:?} with {:?}", q, vars);
            let started = chrono::Local::now();
            let resp =
                graphql_client::reqwest::post_graphql::<Q, _>(&self.client, API_URL, vars.clone())
                    .await?;
            let ended = chrono::Local::now();

            // queries may time out. if that happens throttle the query once and try
            // again, if that fails too we fail for good.
            let resp = match resp.errors {
                None => {
                    // time limit is 10 seconds. if we're well under that, increase
                    // the batch size again.
                    if batch != max_batch && ended - started < chrono::Duration::seconds(8) {
                        batch = (batch + batch / 10 + 1).min(max_batch);
                        log::info!("increasing batch size to {}", batch);
                    }
                    resp
                }
                Some(e) if batch > 1 && e.iter().all(|e| e.message.contains("timeout")) => {
                    log::warn!("throttling query due to timeout error: {:?}", e);
                    // anything larger than 1 seems to be unreliable here
                    batch = 1;
                    log::info!("new batch size: {}", batch);
                    continue;
                }
                Some(e) => bail!("query failed: {:?}", e),
            };

            match resp.data {
                Some(d) => {
                    let (mut items, cursor) = q.process(d)?;
                    result.append(&mut items);
                    match cursor {
                        None => break,
                        cursor => vars = q.change_after(vars, cursor),
                    }
                }
                None => bail!("query returned no data"),
            }
        }

        Ok(result)
    }

    pub async fn get_pulls(&self, repo: &GitHubRepo) -> Result<Vec<store::Pr>> {
        // Currently querying for all PRs ever. In the future, we'll likely want to paginate/be
        // able to pick up where we left off.
        self.query_raw(
            &PullsQuery { since: None },
            pulls_query::Variables {
                owner: repo.owner.clone(),
                name: repo.name.clone(),
                after: None,
                batch: 100,
            },
        ).await?;
        todo!()
    }
}
