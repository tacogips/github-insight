//! MCP (Model Context Protocol) tool implementations for GitInsight
//!
//! This module provides the MCP server interface, exposing GitInsight functionality
//! as tools that can be used by AI assistants and other MCP clients.
//!
//! ## Features
//!
//! - Search issues and pull requests with comprehensive filtering
//! - Get detailed repository and project information
//! - Find related resources through cross-references and semantic similarity
//! - Support for multiple filtering options and hybrid search

use crate::formatter::{
    TimezoneOffset,
    issue::{issue_body_markdown_with_timezone, issue_body_markdown_with_timezone_light},
    project_resource::project_resource_body_markdown_with_timezone,
    pull_request::{
        pull_request_body_markdown_with_timezone, pull_request_body_markdown_with_timezone_light,
    },
};
use crate::github::GitHubClient;
use crate::services::{ProfileService, default_profile_config_dir};
use crate::types::{
    IssueUrl, OutputOption, ProfileName, ProjectUrl, PullRequestUrl, SearchCursorByRepository,
    SearchQuery,
};
use anyhow::Result;
use rmcp::{Error as McpError, ServerHandler, model::*, tool};
use serde_json;

/// Error types specific to tool operations
pub mod error;

/// Tool function implementations organized by functionality
pub mod functions;

/// Wrapper for GitHub code tools exposed through the MCP protocol
#[derive(Clone)]
pub struct GitInsightTools {
    github_token: Option<String>,
    profile_name: Option<ProfileName>,
    #[allow(dead_code)]
    timezone: Option<TimezoneOffset>,
}

const DEFAULT_SEARCH_LIMIT: usize = 30;

fn default_search_limit() -> usize {
    DEFAULT_SEARCH_LIMIT
}

impl GitInsightTools {
    /// Creates a new GitInsightTools instance with optional authentication and profile name
    pub fn new(
        github_token: Option<String>,
        timezone: Option<String>,
        profile_name: Option<ProfileName>,
    ) -> Self {
        let default_timezone = timezone.and_then(|tz| TimezoneOffset::parse(&tz));
        Self {
            github_token,
            profile_name,
            timezone: default_timezone,
        }
    }

    /// Validates profile and extracts project IDs for operations
    ///
    /// Returns Ok(project_ids) if profile exists and has projects,
    /// otherwise returns appropriate error result
    pub fn load_profile_projects(&self) -> Result<Vec<crate::types::ProjectId>, CallToolResult> {
        // Get profile name - use default if none specified
        let profile_name = self.profile_name.clone().unwrap_or_default();

        // Load profile from file
        let config_dir = default_profile_config_dir().map_err(|e| CallToolResult {
            content: vec![Content::text(format!(
                "Failed to get config directory: {}",
                e
            ))],
            is_error: Some(true),
        })?;

        let profile_service = ProfileService::new(config_dir).map_err(|e| CallToolResult {
            content: vec![Content::text(format!(
                "Failed to initialize profile service: {}",
                e
            ))],
            is_error: Some(true),
        })?;

        let project_ids =
            profile_service
                .list_projects(&profile_name)
                .map_err(|e| CallToolResult {
                    content: vec![Content::text(format!(
                        "Failed to load projects from profile '{}': {}",
                        profile_name, e
                    ))],
                    is_error: Some(true),
                })?;

        if project_ids.is_empty() {
            Err(CallToolResult {
                content: vec![Content::text(format!(
                    "No projects found in profile '{}'.",
                    profile_name
                ))],
                is_error: Some(false),
            })
        } else {
            Ok(project_ids)
        }
    }

    /// Validates profile and extracts repository IDs for operations
    ///
    /// Returns Ok(repository_ids) if profile exists and has repositories,
    /// otherwise returns appropriate error result
    pub fn load_profile_repositories(
        &self,
    ) -> Result<Vec<crate::types::RepositoryId>, CallToolResult> {
        // Get profile name - use default if none specified
        let profile_name = self.profile_name.clone().unwrap_or_default();

        // Load profile from file
        let config_dir = default_profile_config_dir().map_err(|e| CallToolResult {
            content: vec![Content::text(format!(
                "Failed to get config directory: {}",
                e
            ))],
            is_error: Some(true),
        })?;

        let profile_service = ProfileService::new(config_dir).map_err(|e| CallToolResult {
            content: vec![Content::text(format!(
                "Failed to initialize profile service: {}",
                e
            ))],
            is_error: Some(true),
        })?;

        let repo_ids = profile_service
            .list_repositories(&profile_name)
            .map_err(|e| CallToolResult {
                content: vec![Content::text(format!(
                    "Failed to load repositories from profile '{}': {}",
                    profile_name, e
                ))],
                is_error: Some(true),
            })?;

        if repo_ids.is_empty() {
            Err(CallToolResult {
                content: vec![Content::text(format!(
                    "No repositories found in profile '{}'.",
                    profile_name
                ))],
                is_error: Some(false),
            })
        } else {
            Ok(repo_ids)
        }
    }

    /// Initializes the GitInsightTools instance with database setup and optional sync
    ///
    /// This method sets up the necessary database connections, profiles, and performs
    /// initial synchronization if requested.
    ///
    /// # Arguments
    ///
    /// # Returns
    /// * `Result<()>` - Success when initialization completes, or error
    pub async fn initialize(&self) -> Result<()> {
        tracing::info!("Initializing GitInsightTools...");

        if let Some(profile_name) = &self.profile_name {
            tracing::info!("Using profile: {}", profile_name);
        } else {
            tracing::info!("Using default profile");
        }

        tracing::info!("GitInsightTools initialization complete");
        Ok(())
    }
}

#[tool(tool_box)]
impl GitInsightTools {
    #[tool(
        description = "Get all project resources. Returns all project resources as markdown array including title, description, resource counts, and timestamps. This tool fetches all resources from the specified project(s) without pagination. Use get_issues_details and get_pull_request_details functions to get more detailed information. Examples: `{}` (all projects), `{\"project_url\": \"https://github.com/users/username/projects/1\"}` (specific project)"
    )]
    async fn get_project_resources(
        &self,

        #[tool(param)]
        #[schemars(
            description = "Optional project URL to fetch resources from. If not provided, fetches all resources from all projects in the profile. Examples: \"https://github.com/users/username/projects/1\""
        )]
        project_url: Option<String>,
    ) -> Result<CallToolResult, McpError> {
        let github_client = GitHubClient::new(self.github_token.clone(), None).map_err(|e| {
            McpError::internal_error(format!("Failed to create GitHub client: {}", e), None)
        })?;

        let mut content_vec = Vec::new();

        if let Some(project_url_str) = project_url {
            // Fetch resources for specific project
            let project_url = ProjectUrl(project_url_str);
            let project_resources =
                functions::project::get_project_resources(&github_client, project_url)
                    .await
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;

            for project_resource in project_resources {
                let formatted = project_resource_body_markdown_with_timezone(
                    &project_resource,
                    self.timezone.as_ref(),
                );
                content_vec.push(Content::text(formatted.0));
            }
        } else {
            // Fetch resources for all projects in the profile
            let project_ids = match self.load_profile_projects() {
                Ok(ids) => ids,
                Err(error_result) => return Ok(error_result),
            };

            let project_resources =
                functions::project::get_multiple_project_resources(&github_client, project_ids)
                    .await
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;

            for project_resource in project_resources {
                let formatted = project_resource_body_markdown_with_timezone(
                    &project_resource,
                    self.timezone.as_ref(),
                );
                content_vec.push(Content::text(formatted.0));
            }
        }

        if content_vec.is_empty() {
            content_vec.push(Content::text("No project resources found.".to_string()));
        }

        Ok(CallToolResult {
            content: content_vec,
            is_error: Some(false),
        })
    }

    #[tool(
        description = "Get issues by their numbers from specified repositories. Returns detailed issue information including comments, formatted as markdown with comprehensive details including title, body, labels, assignees, creation/update dates, and all comments with timestamps."
    )]
    async fn get_issues_details(
        &self,
        #[tool(param)]
        #[schemars(description = "Issue URLs to fetch")]
        issue_urls: Vec<String>,
    ) -> Result<CallToolResult, McpError> {
        let github_client = GitHubClient::new(self.github_token.clone(), None).map_err(|e| {
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
                let formatted = issue_body_markdown_with_timezone(&issue, self.timezone.as_ref());
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

    #[tool(
        description = "Get pull requests by their URLs from specified repositories. Returns detailed pull request information including comments, formatted as markdown with comprehensive details including title, body, labels, assignees, creation/update dates, review status, and all comments with timestamps."
    )]
    async fn get_pull_request_details(
        &self,
        #[tool(param)]
        #[schemars(description = "Pull request URLs to fetch")]
        pull_request_urls: Vec<String>,
    ) -> Result<CallToolResult, McpError> {
        let github_client = GitHubClient::new(self.github_token.clone(), None).map_err(|e| {
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
                    pull_request_body_markdown_with_timezone(&pull_request, self.timezone.as_ref());
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

    #[tool(
        description = "Search across all registered repositories for issues, PRs, and projects. Comprehensive search across multiple resource types. Use get_issues_details and get_pull_request_details functions to get more detailed information."
    )]
    async fn search_across_repositories(
        &self,
        #[tool(param)]
        #[schemars(
            description = "Search query text. Note: Any repo:owner/name specifications in the query will be overridden when searching specific repositories."
        )]
        query: SearchQuery,
        #[tool(param)]
        #[schemars(
            description = "Optional repository ID to search in. If not provided, searches across all repositories in the profile."
        )]
        repository_id: Option<String>,
        #[tool(param)]
        #[schemars(
            description = "Result limit per repository (default 30, max 100). Examples: 10, 50"
        )]
        #[schemars(default = "default_search_limit")]
        limit: Option<usize>,
        #[tool(param)]
        #[schemars(
            description = "Optional search cursors by repository for pagination. Each cursor is associated with a specific repository."
        )]
        cursors: Option<Vec<SearchCursorByRepository>>,
        #[tool(param)]
        #[schemars(
            description = "Output format for search results (light/rich). Light format provides minimal information (title, status, URL, assignees/author, truncated body up to 100 chars, comment count, linked resources), rich format provides comprehensive details (full body, all comments, timestamps, labels, etc.)."
        )]
        #[schemars(default)]
        output_option: Option<OutputOption>,
    ) -> Result<CallToolResult, McpError> {
        let github_client = GitHubClient::new(self.github_token.clone(), None).map_err(|e| {
            McpError::internal_error(format!("Failed to create GitHub client: {}", e), None)
        })?;

        let limit = limit.unwrap_or(DEFAULT_SEARCH_LIMIT);
        let format = output_option.unwrap_or_default();

        let repositories = if let Some(repo_id_str) = repository_id {
            // Search in specific repository
            let repo_id =
                crate::types::RepositoryId::parse_url(&crate::types::RepositoryUrl(repo_id_str))
                    .map_err(|e| {
                        McpError::internal_error(format!("Invalid repository ID: {}", e), None)
                    })?;
            vec![repo_id]
        } else {
            // Search across all repositories in the profile
            match self.load_profile_repositories() {
                Ok(repo_ids) => repo_ids,
                Err(error_result) => return Ok(error_result),
            }
        };

        // Search across repositories
        let search_results = functions::search::search_resources(
            &github_client,
            repositories,
            query,
            Some(limit as u32),
            cursors,
        )
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        // Format results as markdown
        let mut content_vec = Vec::new();

        if search_results.results.is_empty() {
            content_vec.push(Content::text("No results found.".to_string()));
        } else {
            for result in search_results.results {
                let formatted = match result {
                    crate::types::IssueOrPullrequest::Issue(issue) => match format {
                        OutputOption::Light => {
                            issue_body_markdown_with_timezone_light(&issue, self.timezone.as_ref())
                                .0
                        }
                        OutputOption::Rich => {
                            issue_body_markdown_with_timezone(&issue, self.timezone.as_ref()).0
                        }
                    },
                    crate::types::IssueOrPullrequest::PullRequest(pr) => match format {
                        OutputOption::Light => {
                            pull_request_body_markdown_with_timezone_light(
                                &pr,
                                self.timezone.as_ref(),
                            )
                            .0
                        }
                        OutputOption::Rich => {
                            pull_request_body_markdown_with_timezone(&pr, self.timezone.as_ref()).0
                        }
                    },
                };
                content_vec.push(Content::text(formatted));
            }
        }

        // Add cursor information as JSON
        if !search_results.cursors.is_empty() {
            let cursors_json =
                serde_json::to_string_pretty(&search_results.cursors).map_err(|e| {
                    McpError::internal_error(format!("Failed to serialize cursors: {}", e), None)
                })?;
            content_vec.push(Content::text(format!(
                "Next page cursors:\n```json\n{}\n```",
                cursors_json
            )));
        }

        Ok(CallToolResult {
            content: content_vec,
            is_error: Some(false),
        })
    }
}

#[tool(tool_box)]
impl ServerHandler for GitInsightTools {
    /// Provides information about this MCP server
    fn get_info(&self) -> ServerInfo {
        let auth_status = match &self.github_token {
            Some(_) => "Authenticated with GitHub token",
            None => "Not authenticated (rate limits apply)",
        };

        let instructions = format!(
            r#"GitInsight MCP Server - {}

## Overview
GitInsight is a tool for searching GitHub repository data locally. It provides access to issues, pull requests, and comments from GitHub repositories stored in a local database for fast searching.

## Available Tools

### 1. get_project_resources
Get all project resources from specified project(s). Returns all project resources as markdown array including title, description, resource counts, and timestamps. This tool fetches all resources without pagination.

Examples:
```json
// Get all project resources from all projects in profile
{{"name": "get_project_resources", "arguments": {{}}}}

// Get resources from specific project
{{"name": "get_project_resources", "arguments": {{"project_url": "https://github.com/users/username/projects/1"}}}}
```

### 2. get_issues_details
Get issues by their URLs from specified repositories. Returns detailed issue information including comments, formatted as markdown with comprehensive details including title, body, labels, assignees, creation/update dates, and all comments with timestamps.

Examples:
```json
// Get specific issues by URLs
{{"name": "get_issues_details", "arguments": {{"issue_urls": ["https://github.com/rust-lang/rust/issues/12345", "https://github.com/tokio-rs/tokio/issues/5678"]}}}}
```

### 3. get_pull_request_details
Get pull requests by their URLs from specified repositories. Returns detailed pull request information including comments, formatted as markdown with comprehensive details including title, body, labels, assignees, creation/update dates, review status, and all comments with timestamps.

Examples:
```json
// Get specific pull requests by URLs
{{"name": "get_pull_request_details", "arguments": {{"pull_request_urls": ["https://github.com/rust-lang/rust/pull/98765", "https://github.com/tokio-rs/tokio/pull/4321"]}}}}
```

### 4. search_across_repositories
Search across all registered repositories for issues, PRs, and projects. Comprehensive search across multiple resource types with support for specific repository targeting and advanced pagination.

Examples:
```json
// Basic search across all repositories
{{"name": "search_across_repositories", "arguments": {{"query": "memory leak"}}}}

// Search in specific repository
{{"name": "search_across_repositories", "arguments": {{
    "query": "authentication",
    "repository_id": "https://github.com/tokio-rs/tokio",
    "limit": 50
}}}}

// Search with specific output format
{{"name": "search_across_repositories", "arguments": {{
    "query": "async await",
    "output_option": "light",
    "limit": 20
}}}}

// Search with pagination cursors
{{"name": "search_across_repositories", "arguments": {{
    "query": "performance",
    "cursors": [{{"repository_id": "rust-lang/rust", "cursor": "Y3Vyc29yOnYyOpK5"}}]
}}}}
```

## Common Workflows

1. **Repository Search**:
   - Use search_across_repositories to find issues/PRs by keywords across all or specific repositories
   - Support for pagination using cursors for large result sets
   - Choose between light and rich output formats

2. **Specific Resource Access**:
   - Use get_issues_details to get detailed issue information with comments
   - Use get_pull_request_details to get detailed pull request information with comments

3. **Project Management**:
   - Use get_project_resources to access project boards and associated resources
   - Fetch from all projects in profile or specific project URLs

4. **Output Formatting**:
   - Rich format provides comprehensive details including full comments
   - Light format provides minimal information for quick overview
"#,
            auth_status
        );

        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(instructions),
        }
    }
}
