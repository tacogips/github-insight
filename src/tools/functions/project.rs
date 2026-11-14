use anyhow::Result;
use rmcp::Error as McpError;

use crate::{
    github::GitHubClient,
    services::MultiResourceFetcher,
    types::repository::Owner,
    types::{Project, ProjectId, ProjectNumber, ProjectResource, ProjectUrl},
};

pub async fn get_project_resources(
    github_client: &GitHubClient,
    project_url: ProjectUrl,
) -> Result<Vec<ProjectResource>, McpError> {
    // Parse project URL to extract project ID components
    let (owner_str, number, project_type) = ProjectId::parse_url(&project_url).map_err(|e| {
        McpError::invalid_params(format!("Failed to parse project URL: {}", e), None)
    })?;

    // Create ProjectId from parsed components
    let project_id = ProjectId::new(
        Owner::new(owner_str),
        ProjectNumber::new(number),
        project_type,
    );

    // Create MultiResourceFetcher and fetch project resources
    let fetcher = MultiResourceFetcher::new(github_client.clone());
    fetcher
        .fetch_project_resources(project_id)
        .await
        .map_err(|e| {
            McpError::internal_error(format!("Failed to fetch project resources: {}", e), None)
        })
}

pub async fn get_multiple_project_resources(
    github_client: &GitHubClient,
    project_ids: Vec<ProjectId>,
) -> Result<Vec<ProjectResource>, McpError> {
    let fetcher = MultiResourceFetcher::new(github_client.clone());
    let mut all_resources = Vec::new();

    for project_id in project_ids {
        match fetcher.fetch_project_resources(project_id.clone()).await {
            Ok(project_resources) => {
                all_resources.extend(project_resources);
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to fetch project resources for {}: {}",
                    project_id,
                    e
                );
            }
        }
    }

    Ok(all_resources)
}

pub async fn get_projects_details(
    github_client: &GitHubClient,
    project_urls: Vec<ProjectUrl>,
) -> Result<Vec<Project>, McpError> {
    let fetcher = MultiResourceFetcher::new(github_client.clone());
    let mut all_projects = Vec::new();

    for project_url in project_urls {
        // Parse project URL to extract project ID components
        let (owner_str, number, project_type) =
            ProjectId::parse_url(&project_url).map_err(|e| {
                McpError::invalid_params(
                    format!("Failed to parse project URL {}: {}", project_url, e),
                    None,
                )
            })?;

        // Create ProjectId from parsed components
        let project_id = ProjectId::new(
            Owner::new(owner_str),
            ProjectNumber::new(number),
            project_type,
        );

        // Fetch project details
        match fetcher.fetch_project(project_id.clone()).await {
            Ok(project) => {
                all_projects.push(project);
            }
            Err(e) => {
                tracing::warn!("Failed to fetch project details for {}: {}", project_id, e);
            }
        }
    }

    Ok(all_projects)
}
