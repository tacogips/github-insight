use crate::formatter::{TimezoneOffset, pull_request::pull_request_body_markdown_with_timezone};
use crate::github::GitHubClient;
use crate::tools::functions;
use crate::types::PullRequestUrl;
use anyhow::Result;
use rmcp::{Error as McpError, model::*};

/// Get pull requests by their URLs from specified repositories
///
/// Returns detailed pull request information including comments, formatted as markdown
/// with comprehensive details including title, body, labels, assignees,
/// creation/update dates, review status, and all comments with timestamps.
pub async fn get_pull_request_details(
    github_token: &Option<String>,
    timezone: &Option<TimezoneOffset>,
    pull_request_urls: Vec<String>,
) -> Result<CallToolResult, McpError> {
    let github_client = GitHubClient::new(github_token.clone(), None).map_err(|e| {
        McpError::internal_error(format!("Failed to create GitHub client: {}", e), None)
    })?;

    // Convert strings to PullRequestUrl
    let pull_request_urls: Vec<PullRequestUrl> =
        pull_request_urls.into_iter().map(PullRequestUrl).collect();

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
