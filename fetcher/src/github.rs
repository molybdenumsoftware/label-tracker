const API_URL: &str = "https://api.github.com/graphql";

pub struct GitHub {
    client: reqwest::Client
}


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
