use anyhow::Result;
use futures_util::stream::StreamExt as _;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GithubProvider {
    pub owner: String,
    pub repo: String,
    pub ref_: String,
    pub path: String,
}

impl GithubProvider {
    pub async fn fetch_artifactory(&self) -> Result<Vec<u8>> {
        let client = Client::new();
        let url = format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}",
            self.owner, self.repo, self.ref_, self.path
        );

        let response = client.get(&url).send().await?;
        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }

    pub async fn download_package(&self, package_path: &str, destination: &PathBuf) -> Result<()> {
        let client = Client::new();

        // Construct the raw GitHub URL for the package
        let url = if package_path.starts_with("http") {
            // If it's already a full URL, use it directly
            package_path.to_string()
        } else {
            // Otherwise construct GitHub raw URL
            format!(
                "https://raw.githubusercontent.com/{}/{}/{}/{}",
                self.owner, self.repo, self.ref_, package_path
            )
        };

        // Start the download with a progress bar
        let response = client.get(&url).send().await?;
        let total_size = response.content_length().unwrap_or(0);

        let pb = indicatif::ProgressBar::new(total_size);
        pb.set_style(indicatif::ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("#>-"));

        // Stream the download with progress updates
        let mut file = tokio::fs::File::create(destination).await?;
        let mut stream = response.bytes_stream();

        let mut downloaded: u64 = 0;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await?;
            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }

        pb.finish_with_message("Download complete");
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GithubProviderError {
    #[error("Failed to fetch artifactory: {0}")]
    FetchError(#[from] reqwest::Error),
    #[error("Failed to write package: {0}")]
    IoError(#[from] std::io::Error),
}
