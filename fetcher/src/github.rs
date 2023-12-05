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
