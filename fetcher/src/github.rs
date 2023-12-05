const API_URL: &str = "https://api.github.com/graphql";

pub struct GitHub {
    client: reqwest::Client
}
