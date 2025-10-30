use crate::formatter::{TimezoneOffset, issue::issue_body_markdown_with_timezone};
use crate::github::GitHubClient;
use crate::tools::functions;
use crate::types::IssueUrl;
use anyhow::Result;
use rmcp::{Error as McpError, model::*};

/// Get issues by their URLs from specified repositories
///
/// Returns detailed issue information including comments, formatted as markdown
/// with comprehensive details including title, body, labels, assignees,
/// creation/update dates, and all comments with timestamps.
pub async fn get_issues_details(
    github_token: &Option<String>,
    timezone: &Option<TimezoneOffset>,
    issue_urls: Vec<String>,
) -> Result<CallToolResult, McpError> {
    let github_client = GitHubClient::new(github_token.clone(), None).map_err(|e| {
        McpError::internal_error(format!("Failed to create GitHub client: {}", e), None)
    })?;

    // Convert strings to IssueUrl
    let issue_urls: Vec<IssueUrl> = issue_urls.into_iter().map(IssueUrl).collect();

    // Fetch issues using the existing function
    let issues_by_repo = functions::issue::get_issues_details(&github_client, issue_urls)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    // Format all issues as markdown
    let mut content_vec = Vec::new();

    for (_repo_id, issues) in issues_by_repo {
        for issue in issues {
            let formatted = issue_body_markdown_with_timezone(&issue, timezone.as_ref());
            content_vec.push(Content::text(formatted.0));
        }
    }

    if content_vec.is_empty() {
        content_vec.push(Content::text(
            "No issues found for the provided URLs.".to_string(),
        ));
    }

    Ok(CallToolResult {
        content: content_vec,
        is_error: Some(false),
    })
}
