use crate::formatter::{TimezoneOffset, repository::repository_body_markdown_with_timezone};
use crate::github::GitHubClient;
use crate::tools::functions;
use anyhow::Result;
use rmcp::{Error as McpError, model::*};

/// Get repository details by URLs
///
/// Returns detailed repository information formatted as markdown with comprehensive
/// metadata including URL, description, default branch, mentionable users, labels,
/// milestones, releases (with configurable limit), and timestamps.
pub async fn get_repository_details(
    github_token: &Option<String>,
    timezone: &Option<TimezoneOffset>,
    repository_urls: Vec<String>,
    showing_release_limit: Option<usize>,
    showing_milestone_limit: Option<usize>,
) -> Result<CallToolResult, McpError> {
    let github_client = GitHubClient::new(github_token.clone(), None).map_err(|e| {
        McpError::internal_error(format!("Failed to create GitHub client: {}", e), None)
    })?;

    // Check if repository_urls is empty and return error
    if repository_urls.is_empty() {
        return Err(McpError::invalid_request(
            "repository_urls cannot be empty. Please provide at least one repository URL."
                .to_string(),
            None,
        ));
    }

    let repository_urls = repository_urls
        .into_iter()
        .map(crate::types::RepositoryUrl)
        .collect::<Vec<_>>();

    // Fetch repositories using the multiple repositories function
    let repositories =
        functions::repository::get_multiple_repository_details(&github_client, repository_urls)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    // Format all repositories as markdown
    let mut content_vec = Vec::new();

    for repository in repositories {
        let formatted = repository_body_markdown_with_timezone(
            &repository,
            timezone.as_ref(),
            showing_release_limit,
            showing_milestone_limit,
        );
        content_vec.push(Content::text(formatted.0));
    }

    if content_vec.is_empty() {
        content_vec.push(Content::text(
            "No repositories found for the provided URLs.".to_string(),
        ));
    }

    Ok(CallToolResult {
        content: content_vec,
        is_error: Some(false),
    })
}
