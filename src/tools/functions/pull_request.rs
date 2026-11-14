use anyhow::Result;
use std::collections::BTreeMap;

use crate::github::GitHubClient;
use crate::services::MultiResourceFetcher;
use crate::types::{PullRequest, PullRequestId, PullRequestNumber, PullRequestUrl, RepositoryId};

pub async fn get_pull_requests_details(
    github_client: &GitHubClient,
    pull_request_urls: Vec<PullRequestUrl>,
) -> Result<BTreeMap<RepositoryId, Vec<PullRequest>>> {
    // Convert URLs to PullRequestIds and group by repository
    let mut pull_request_ids_by_repo: BTreeMap<RepositoryId, Vec<PullRequestNumber>> =
        BTreeMap::new();

    for url in pull_request_urls {
        match PullRequestId::parse_url(&url) {
            Ok(pull_request_id) => {
                let pull_request_number = PullRequestNumber::new(pull_request_id.number);
                pull_request_ids_by_repo
                    .entry(pull_request_id.git_repository)
                    .or_default()
                    .push(pull_request_number);
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to parse issue URL {}: {}", url, e));
            }
        }
    }

    // Convert to the format expected by fetch_issues
    let pull_request_ids_of_repositories: Vec<(RepositoryId, Vec<PullRequestNumber>)> =
        pull_request_ids_by_repo.into_iter().collect();

    // Create MultiResourceFetcher and fetch issues
    let fetcher = MultiResourceFetcher::new(github_client.clone());
    fetcher
        .fetch_pull_requests(pull_request_ids_of_repositories)
        .await
}
