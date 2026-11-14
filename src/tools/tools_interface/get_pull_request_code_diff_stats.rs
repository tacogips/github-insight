use crate::formatter::pull_request_file_stats::pull_request_file_stats_markdown;
use crate::github::GitHubClient;
use crate::tools::functions;
use crate::types::PullRequestUrl;
use anyhow::Result;
use rmcp::{Error as McpError, model::*};

/// Get pull request file statistics by their URLs
///
/// Returns file-level change statistics (additions, deletions, changes) for each
/// pull request without the actual diff content. Use this for quick overview of
/// changed files and their modification counts.
pub async fn get_pull_request_code_diff_stats(
    github_token: &Option<String>,
    pull_request_urls: Vec<String>,
) -> Result<CallToolResult, McpError> {
    let github_client = GitHubClient::new(github_token.clone(), None).map_err(|e| {
        McpError::internal_error(format!("Failed to create GitHub client: {}", e), None)
    })?;

    // Convert strings to PullRequestUrl
    let pull_request_urls: Vec<PullRequestUrl> =
        pull_request_urls.into_iter().map(PullRequestUrl).collect();

    // Fetch pull request file stats using the new function
    let files_by_repo =
        functions::pull_request::get_pull_request_files_stats(&github_client, pull_request_urls)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    // Format all file stats as markdown using the formatter
    let mut content_vec = Vec::new();

    for (repo_id, pr_files) in files_by_repo {
        for (pr_number, files) in pr_files {
            let formatted = pull_request_file_stats_markdown(&repo_id, pr_number, &files);
            content_vec.push(Content::text(formatted.0));
        }
    }

    if content_vec.is_empty() {
        content_vec.push(Content::text(
            "No pull request file statistics found for the provided URLs.".to_string(),
        ));
    }

    Ok(CallToolResult {
        content: content_vec,
        is_error: Some(false),
    })
}
