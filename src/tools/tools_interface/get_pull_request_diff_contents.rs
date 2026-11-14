use crate::github::GitHubClient;
use crate::tools::functions;
use crate::types::PullRequestUrl;
use anyhow::Result;
use rmcp::{Error as McpError, model::*};

/// Get the diff content of a specific file from a pull request
///
/// Returns the unified diff patch for the specified file. Supports optional
/// skip/limit filtering to retrieve specific portions of the diff.
pub async fn get_pull_request_diff_contents(
    github_token: &Option<String>,
    pull_request_url: String,
    file_path: String,
    skip: Option<u32>,
    limit: Option<u32>,
) -> Result<CallToolResult, McpError> {
    let github_client = GitHubClient::new(github_token.clone(), None).map_err(|e| {
        McpError::internal_error(format!("Failed to create GitHub client: {}", e), None)
    })?;

    // Convert string to PullRequestUrl
    let pull_request_url = PullRequestUrl(pull_request_url);

    // Fetch the diff content
    let diff_content = functions::pull_request::get_pull_request_diff_contents(
        &github_client,
        pull_request_url,
        file_path.clone(),
        skip,
        limit,
    )
    .await
    .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    // Format as markdown code block
    let formatted = format!(
        "## Diff for file: {}\n\n```diff\n{}\n```",
        file_path, diff_content
    );

    Ok(CallToolResult {
        content: vec![Content::text(formatted)],
        is_error: Some(false),
    })
}
