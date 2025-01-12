use serde::{Deserialize, Serialize};

pub(crate) mod github;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Provider {
    pub name: String,
    pub source: ProviderSource,
    pub provider_handler_version: u8,
}

/// The source of the provider.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ProviderSource {
    Github(github::GithubProvider),
}
