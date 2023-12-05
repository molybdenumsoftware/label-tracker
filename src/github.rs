use std::{collections::BTreeSet, fmt::Debug};

use anyhow::{bail, Result};
use chrono::Duration;
use graphql_client::{reqwest::post_graphql_blocking as post_graphql, GraphQLQuery};

use crate::state::{DateTime, Issue, PullRequest, HTML, URI};

const API_URL: &str = "https://api.github.com/graphql";

type Cursor = String;
type GitObjectID = String;

pub struct Github {
    client: reqwest::blocking::Client,
    owner: String,
    repo: String,
    label: String,
}

trait ChunkedQuery: GraphQLQuery {
    type Item;

    fn change_after(&self, v: Self::Variables, after: Option<String>) -> Self::Variables;
    fn set_batch(&self, batch: i64, v: Self::Variables) -> Self::Variables;

    fn process(&self, d: Self::ResponseData) -> Result<(Vec<Self::Item>, Option<Cursor>)>;
}

#[derive(Debug, GraphQLQuery)]
#[graphql(
    schema_path = "vendor/github.com/schema.docs.graphql",
    query_path = "src/issues.graphql",
    response_derives = "Debug",
    variables_derives = "Clone,Debug"
)]
pub struct IssuesQuery;

impl ChunkedQuery for IssuesQuery {
    type Item = Issue;

    fn change_after(&self, v: Self::Variables, after: Option<String>) -> Self::Variables {
        Self::Variables { after, ..v }
    }
    fn set_batch(&self, batch: i64, v: Self::Variables) -> Self::Variables {
        Self::Variables { batch, ..v }
    }

    fn process(&self, d: Self::ResponseData) -> Result<(Vec<Self::Item>, Option<Cursor>)> {
        debug!("rate limits: {:?}", d.rate_limit);
        let issues = match d.repository {
            Some(r) => r.issues,
            None => bail!("query returned no repo"),
        };
        // deliberately ignore all nulls. no idea why the schema doesn't make
        // all of these links mandatory, having them nullable makes no sense.
        let infos = issues
            .edges
            .unwrap_or_default()
            .into_iter()
            .filter_map(|e| e?.node)
            .map(|n| Issue {
                id: n.id,
                title: n.title,
                is_open: !n.closed,
                body: n.body_html,
                last_update: n.updated_at,
                url: n.url,
            })
            .collect();
        let cursor = if issues.page_info.has_next_page {
            issues.page_info.end_cursor
        } else {
            None
        };
        Ok((infos, cursor))
    }
}

#[derive(Debug, GraphQLQuery)]
#[graphql(
    schema_path = "vendor/github.com/schema.docs.graphql",
    query_path = "src/pulls.graphql",
    response_derives = "Debug",
    variables_derives = "Clone,Debug"
)]
pub struct PullsQuery {
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

    fn query_raw<Q>(&self, q: &Q, mut vars: <Q as GraphQLQuery>::Variables) -> Result<Vec<Q::Item>>
    where
        Q: ChunkedQuery + Debug,
        Q::Variables: Clone + Debug,
    {
        let mut result = vec![];
        let max_batch = 100;
        let mut batch = max_batch;

        loop {
            vars = q.set_batch(batch, vars);

            debug!("running query {:?} with {:?}", q, vars);
            let started = chrono::Local::now();
            let resp = post_graphql::<Q, _>(&self.client, API_URL, vars.clone())?;
            let ended = chrono::Local::now();

            // queries may time out. if that happens throttle the query once and try
            // again, if that fails too we fail for good.
            let resp = match resp.errors {
                None => {
                    // time limit is 10 seconds. if we're well under that, increase
                    // the batch size again.
                    if batch != max_batch && ended - started < Duration::seconds(8) {
                        batch = (batch + batch / 10 + 1).min(max_batch);
                        info!("increasing batch size to {}", batch);
                    }
                    resp
                }
                Some(e) if batch > 1 && e.iter().all(|e| e.message.contains("timeout")) => {
                    warn!("throttling query due to timeout error: {:?}", e);
                    // anything larger than 1 seems to be unreliable here
                    batch = 1;
                    info!("new batch size: {}", batch);
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

    pub fn query_issues(&self, since: Option<DateTime>) -> Result<Vec<Issue>> {
        self.query_raw(
            &IssuesQuery,
            issues_query::Variables {
                owner: self.owner.clone(),
                name: self.repo.clone(),
                label: self.label.clone(),
                after: None,
                since,
                batch: 100,
            },
        )
    }

    pub fn query_pulls(&self, since: Option<DateTime>) -> Result<Vec<PullRequest>> {
        self.query_raw(
            &PullsQuery { since },
            pulls_query::Variables {
                owner: self.owner.clone(),
                name: self.repo.clone(),
                label: self.label.clone(),
                after: None,
                batch: 100,
            },
        )
    }
}
