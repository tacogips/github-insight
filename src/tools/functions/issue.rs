use anyhow::Result;
use std::collections::BTreeMap;

use crate::github::GitHubClient;
use crate::services::MultiResourceFetcher;
use crate::types::{Issue, IssueId, IssueNumber, IssueUrl, RepositoryId};

pub async fn get_issues_details(
    github_client: &GitHubClient,
    issue_urls: Vec<IssueUrl>,
) -> Result<BTreeMap<RepositoryId, Vec<Issue>>> {
    // Convert URLs to IssueIds and group by repository
    let mut issue_ids_by_repo: BTreeMap<RepositoryId, Vec<IssueNumber>> = BTreeMap::new();

    for url in issue_urls {
        match IssueId::parse_url(&url) {
            Ok(issue_id) => {
                let issue_number = IssueNumber::new(issue_id.number);
                issue_ids_by_repo
                    .entry(issue_id.git_repository)
                    .or_default()
                    .push(issue_number);
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to parse issue URL {}: {}", url, e));
            }
        }
    }

    // Convert to the format expected by fetch_issues
    let issue_ids_of_repositories: Vec<(RepositoryId, Vec<IssueNumber>)> =
        issue_ids_by_repo.into_iter().collect();

    // Create MultiResourceFetcher and fetch issues
    let fetcher = MultiResourceFetcher::new(github_client.clone());
    fetcher.fetch_issues(issue_ids_of_repositories).await
}
