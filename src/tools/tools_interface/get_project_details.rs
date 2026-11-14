use crate::formatter::{TimezoneOffset, project::project_body_markdown_with_timezone};
use crate::github::GitHubClient;
use crate::tools::functions;
use crate::types::ProjectUrl;
use anyhow::Result;
use rmcp::{Error as McpError, model::*};

/// Get project details by their URLs
///
/// Returns detailed project information formatted as markdown with comprehensive
/// metadata including title, description, creation/update dates, project node ID,
/// and other project properties. The project node ID can be used for project updates.
pub async fn get_project_details(
    github_token: &Option<String>,
    timezone: &Option<TimezoneOffset>,
    project_urls: Vec<String>,
) -> Result<CallToolResult, McpError> {
    let github_client = GitHubClient::new(github_token.clone(), None).map_err(|e| {
        McpError::internal_error(format!("Failed to create GitHub client: {}", e), None)
    })?;

    // Convert strings to ProjectUrl
    let project_urls: Vec<ProjectUrl> = project_urls.into_iter().map(ProjectUrl).collect();

    // Fetch projects using the existing function
    let projects = functions::project::get_projects_details(&github_client, project_urls)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    // Format all projects as markdown
    let mut content_vec = Vec::new();

    for project in projects {
        let formatted = project_body_markdown_with_timezone(&project, timezone.as_ref());
        content_vec.push(Content::text(formatted.0));
    }

    if content_vec.is_empty() {
        content_vec.push(Content::text(
            "No projects found for the provided URLs.".to_string(),
        ));
    }

    Ok(CallToolResult {
        content: content_vec,
        is_error: Some(false),
    })
}
