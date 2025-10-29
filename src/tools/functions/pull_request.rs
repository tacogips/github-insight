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

pub async fn get_pull_request_code_diffs(
    github_client: &GitHubClient,
    pull_request_urls: Vec<PullRequestUrl>,
) -> Result<BTreeMap<RepositoryId, Vec<(PullRequestNumber, String)>>> {
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
                return Err(anyhow::anyhow!(
                    "Failed to parse pull request URL {}: {}",
                    url,
                    e
                ));
            }
        }
    }

    // Convert to the format expected by fetch_pull_request_diffs
    let pull_request_ids_of_repositories: Vec<(RepositoryId, Vec<PullRequestNumber>)> =
        pull_request_ids_by_repo.into_iter().collect();

    // Create MultiResourceFetcher and fetch diffs
    let fetcher = MultiResourceFetcher::new(github_client.clone());
    fetcher
        .fetch_pull_request_diffs(pull_request_ids_of_repositories)
        .await
}

pub async fn get_pull_request_files_stats(
    github_client: &GitHubClient,
    pull_request_urls: Vec<PullRequestUrl>,
) -> Result<BTreeMap<RepositoryId, Vec<(PullRequestNumber, Vec<crate::types::PullRequestFile>)>>> {
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
                return Err(anyhow::anyhow!(
                    "Failed to parse pull request URL {}: {}",
                    url,
                    e
                ));
            }
        }
    }

    // Convert to the format expected by fetch_pull_request_files_stats
    let pull_request_ids_of_repositories: Vec<(RepositoryId, Vec<PullRequestNumber>)> =
        pull_request_ids_by_repo.into_iter().collect();

    // Create MultiResourceFetcher and fetch file stats
    let fetcher = MultiResourceFetcher::new(github_client.clone());
    fetcher
        .fetch_pull_request_files_stats(pull_request_ids_of_repositories)
        .await
}

/// Get the diff content of a specific file from a pull request
///
/// # Arguments
///
/// * `github_client` - GitHub client instance
/// * `pull_request_url` - Pull request URL
/// * `file_path` - File path within the repository
/// * `skip` - Optional number of lines to skip from the beginning
/// * `limit` - Optional maximum number of lines to return
///
/// # Returns
///
/// Returns the diff content as a String. If skip/limit is specified, only returns
/// the requested portion of the diff.
pub async fn get_pull_request_diff_contents(
    github_client: &GitHubClient,
    pull_request_url: PullRequestUrl,
    file_path: String,
    skip: Option<u32>,
    limit: Option<u32>,
) -> Result<String> {
    // Parse URL to get repository and PR number
    let pull_request_id = PullRequestId::parse_url(&pull_request_url).map_err(|e| {
        anyhow::anyhow!(
            "Failed to parse pull request URL {}: {}",
            pull_request_url,
            e
        )
    })?;

    let pull_request_number = PullRequestNumber::new(pull_request_id.number);

    // Fetch the file content (patch) from the pull request
    let patch = github_client
        .fetch_pull_request_file_content(
            pull_request_id.git_repository,
            pull_request_number,
            &file_path,
        )
        .await?
        .ok_or_else(|| {
            anyhow::anyhow!(
                "No patch content found for file '{}' in pull request",
                file_path
            )
        })?;

    // If no skip/limit is specified, return the entire patch
    if skip.is_none() && limit.is_none() {
        return Ok(patch);
    }

    // Filter lines based on skip and limit
    let lines: Vec<&str> = patch.lines().collect();
    let skip_count = skip.unwrap_or(0) as usize;

    // Validate skip
    if skip_count > lines.len() {
        return Err(anyhow::anyhow!(
            "skip {} exceeds total lines {}",
            skip_count,
            lines.len()
        ));
    }

    // Calculate the range
    let start_idx = skip_count;
    let end_idx = if let Some(limit_val) = limit {
        (start_idx + limit_val as usize).min(lines.len())
    } else {
        lines.len()
    };

    let filtered_lines = &lines[start_idx..end_idx];
    Ok(filtered_lines.join("\n"))
}
