use crate::formatter::{
    TimezoneOffset,
    project_resource::{
        project_resource_body_markdown_with_timezone,
        project_resource_body_markdown_with_timezone_light,
    },
};
use crate::github::GitHubClient;
use crate::tools::functions;
use crate::types::{OutputOption, ProjectUrl};
use anyhow::Result;
use rmcp::{Error as McpError, model::*};

/// Get all project resources from specified project(s)
///
/// Returns all project resources as markdown array including title, description,
/// resource counts, and timestamps. Each project resource includes field IDs that
/// can be used for project field updates. This tool fetches all resources without pagination.
pub async fn get_project_resources(
    github_token: &Option<String>,
    timezone: &Option<TimezoneOffset>,
    project_urls: Vec<String>,
    output_option: Option<String>,
) -> Result<CallToolResult, McpError> {
    let github_client = GitHubClient::new(github_token.clone(), None).map_err(|e| {
        McpError::internal_error(format!("Failed to create GitHub client: {}", e), None)
    })?;

    // Check if project_urls is empty and return error
    if project_urls.is_empty() {
        return Err(McpError::invalid_request(
            "project_urls cannot be empty. Please provide at least one project URL. To get projects from the current profile, use list_project_urls_in_current_profile to get project URLs and pass them to this parameter.".to_string(),
            None,
        ));
    }

    // Parse output format option, defaulting to rich
    let format = if let Some(option_str) = output_option {
        option_str
            .parse::<OutputOption>()
            .unwrap_or(OutputOption::Rich)
    } else {
        OutputOption::Rich
    };

    let mut content_vec = Vec::new();

    // Convert strings to ProjectId
    let mut project_ids = Vec::new();
    for project_url_str in project_urls {
        let project_url = ProjectUrl(project_url_str);
        let (owner_str, number, project_type) = crate::types::ProjectId::parse_url(&project_url)
            .map_err(|e| {
                McpError::invalid_params(format!("Failed to parse project URL: {}", e), None)
            })?;

        let project_id = crate::types::ProjectId::new(
            crate::types::repository::Owner::new(owner_str),
            crate::types::ProjectNumber::new(number),
            project_type,
        );
        project_ids.push(project_id);
    }

    // Fetch resources for specified projects
    let project_resources =
        functions::project::get_multiple_project_resources(&github_client, project_ids)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    for project_resource in project_resources {
        let formatted = match format {
            OutputOption::Light => project_resource_body_markdown_with_timezone_light(
                &project_resource,
                timezone.as_ref(),
            ),
            OutputOption::Rich => {
                project_resource_body_markdown_with_timezone(&project_resource, timezone.as_ref())
            }
        };
        content_vec.push(Content::text(formatted.0));
    }

    if content_vec.is_empty() {
        content_vec.push(Content::text("No project resources found.".to_string()));
    }

    Ok(CallToolResult {
        content: content_vec,
        is_error: Some(false),
    })
}
