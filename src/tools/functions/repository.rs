use anyhow::Result;
use futures::stream::{self, StreamExt};

use crate::github::GitHubClient;
use crate::services::MultiResourceFetcher;
use crate::types::{GithubRepository, RepositoryId, RepositoryUrl};

pub async fn get_multiple_repository_details(
    github_client: &GitHubClient,
    repository_urls: Vec<RepositoryUrl>,
) -> Result<Vec<GithubRepository>> {
    // Parse URLs to repository IDs first
    let repository_ids: Result<Vec<RepositoryId>, anyhow::Error> = repository_urls
        .iter()
        .map(|url| {
            RepositoryId::parse_url(url)
                .map_err(|e| anyhow::anyhow!("Failed to parse repository URL {}: {}", url, e))
        })
        .collect();

    let repository_ids = repository_ids?;

    // Fetch repositories concurrently
    let fetch_futures = repository_ids.into_iter().map(|repo_id| {
        let github_client = github_client.clone();
        async move {
            let fetcher = MultiResourceFetcher::new(github_client);
            fetcher.fetch_repository(repo_id).await
        }
    });

    let results: Vec<Result<GithubRepository>> = stream::iter(fetch_futures)
        .buffer_unordered(10) // Process up to 10 repositories concurrently
        .collect()
        .await;

    // Collect successful results and log errors
    let repositories: Vec<GithubRepository> = results
        .into_iter()
        .filter_map(|result| match result {
            Ok(repo) => Some(repo),
            Err(e) => {
                tracing::warn!("Failed to fetch repository: {}", e);
                None
            }
        })
        .collect();

    Ok(repositories)
}
