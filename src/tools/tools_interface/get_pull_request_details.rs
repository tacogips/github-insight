use crate::formatter::{TimezoneOffset, pull_request::pull_request_body_markdown_with_timezone};
use crate::git;
use crate::github::GitHubClient;
use crate::tools::functions;
use crate::types::{IssueOrPullrequest, PullRequestUrl, RepositoryId, RepositoryUrl, SearchQuery};
use anyhow::Result;
use rmcp::{Error as McpError, model::*};

/// Find pull request URLs by searching for a branch name in a repository
///
/// # Arguments
///
/// * `github_client` - GitHub client for API access
/// * `working_directory` - Optional working directory path (defaults to current directory)
/// * `branch_name` - Optional branch name (defaults to current branch if not specified)
///
/// # Returns
///
/// * `Result<Vec<String>, McpError>` - List of PR URLs or error
async fn find_pull_request_urls_by_branch(
    github_client: &GitHubClient,
    working_directory: Option<&str>,
    branch_name: Option<&str>,
) -> Result<Vec<String>, McpError> {
    let work_dir = working_directory.unwrap_or(".");

    // Determine branch name: use specified branch or detect current branch
    let (target_branch, repo_url) = if let Some(branch) = branch_name {
        // Use specified branch name, but still need repository URL from git
        let repo_url = git::get_remote_url(work_dir).map_err(|e| {
            McpError::invalid_request(format!("Failed to get Git repository URL: {}", e), None)
        })?;
        (branch.to_string(), repo_url)
    } else {
        // Get current branch and repository info
        let repo_info = git::get_repository_info(work_dir).map_err(|e| {
            McpError::invalid_request(
                format!("Failed to get Git repository information: {}", e),
                None,
            )
        })?;
        (repo_info.current_branch, repo_info.repository_url)
    };

    // Parse repository URL to get owner and name
    let repo_url_type = RepositoryUrl(repo_url.clone());
    let repo_id = RepositoryId::parse_url(&repo_url_type).map_err(|e| {
        McpError::invalid_request(format!("Failed to parse repository URL: {}", e), None)
    })?;

    // Search for pull requests with the specified branch as head
    let search_query = SearchQuery::new(format!("is:pr head:{}", target_branch));

    let search_results = crate::tools::functions::search::search_resources(
        github_client,
        vec![repo_id.clone()],
        search_query,
        Some(10), // Limit to 10 results
        None,     // No cursors
    )
    .await
    .map_err(|e| {
        McpError::internal_error(format!("Failed to search for pull requests: {}", e), None)
    })?;

    // Extract PR URLs from search results
    let mut found_urls = Vec::new();
    for result in search_results.results {
        if let IssueOrPullrequest::PullRequest(pr) = result {
            found_urls.push(pr.pull_request_id.url());
        }
    }

    if found_urls.is_empty() {
        return Err(McpError::invalid_request(
            format!(
                "No pull requests found for branch '{}' in repository '{}'.",
                target_branch, repo_url
            ),
            None,
        ));
    }

    Ok(found_urls)
}

/// Get pull requests by their URLs from specified repositories or by branch name
///
/// This tool supports two modes of operation:
/// 1. URL mode: Provide pull_request_urls to get PR details by direct URLs
/// 2. Branch mode: Provide branch_name (and optionally working_directory) to find PRs for a branch
///
/// Returns detailed pull request information including comments, formatted as markdown
/// with comprehensive details including title, body, labels, assignees,
/// creation/update dates, review status, and all comments with timestamps.
///
/// # Arguments
///
/// * `github_token` - Optional GitHub authentication token
/// * `timezone` - Optional timezone for formatting dates
/// * `pull_request_urls` - Optional list of PR URLs (for URL mode)
/// * `working_directory` - Optional working directory path (for branch mode, defaults to current directory)
/// * `branch_name` - Optional branch name (for branch mode, defaults to current branch if not specified)
///
/// # Returns
///
/// * `Result<CallToolResult, McpError>` - Tool result with PR information or error
pub async fn get_pull_request_details(
    github_token: &Option<String>,
    timezone: &Option<TimezoneOffset>,
    pull_request_urls: Option<Vec<String>>,
    working_directory: Option<String>,
    branch_name: Option<String>,
) -> Result<CallToolResult, McpError> {
    let github_client = GitHubClient::new(github_token.clone(), None).map_err(|e| {
        McpError::internal_error(format!("Failed to create GitHub client: {}", e), None)
    })?;

    // Determine which mode to use and get PR URLs
    let pr_urls = if let Some(urls) = pull_request_urls {
        // URL mode: use provided URLs directly
        urls
    } else {
        // Branch mode: detect branch and search for PRs
        // If all parameters are None, default to current directory and current branch
        match find_pull_request_urls_by_branch(
            &github_client,
            working_directory.as_deref(),
            branch_name.as_deref(),
        )
        .await
        {
            Ok(urls) => urls,
            Err(e) => {
                return Ok(CallToolResult {
                    content: vec![Content::text(e.to_string())],
                    is_error: Some(false),
                });
            }
        }
    };

    // Convert strings to PullRequestUrl
    let pull_request_urls: Vec<PullRequestUrl> = pr_urls.into_iter().map(PullRequestUrl).collect();

    // Fetch pull requests using the existing function
    let pull_requests_by_repo =
        functions::pull_request::get_pull_requests_details(&github_client, pull_request_urls)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    // Format all pull requests as markdown
    let mut content_vec = Vec::new();

    for (_repo_id, pull_requests) in pull_requests_by_repo {
        for pull_request in pull_requests {
            let formatted =
                pull_request_body_markdown_with_timezone(&pull_request, timezone.as_ref());
            content_vec.push(Content::text(formatted.0));
        }
    }

    if content_vec.is_empty() {
        content_vec.push(Content::text(
            "No pull requests found for the provided URLs.".to_string(),
        ));
    }

    Ok(CallToolResult {
        content: content_vec,
        is_error: Some(false),
    })
}
