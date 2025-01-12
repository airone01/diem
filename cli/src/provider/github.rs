use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GithubProvider {
    pub owner: String,
    pub repo: String,
    pub ref_: String,
    pub path: String,
}
