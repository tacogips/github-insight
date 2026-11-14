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
    project_resource::{
        project_resource_body_markdown_with_timezone,
        project_resource_body_markdown_with_timezone_light,
    },
    pull_request::{
        pull_request_body_markdown_with_timezone, pull_request_body_markdown_with_timezone_light,
    },
    repository::repository_body_markdown_with_timezone,
};
use crate::github::GitHubClient;
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
const DEFAULT_SEARCH_QUERY: &str = "state:open";

fn default_search_limit() -> usize {
    DEFAULT_SEARCH_LIMIT
}

fn default_search_query() -> String {
    DEFAULT_SEARCH_QUERY.to_string()
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
        description = "Get all project resources. Returns all project resources as markdown array including title, description, resource counts, and timestamps. This tool fetches all resources from the specified project(s) without pagination. Each project resource includes field IDs that can be used for project field updates. Use get_issues_details and get_pull_request_details functions to get more detailed information. Examples: `{}` (all projects), `{\"project_url\": \"https://github.com/users/username/projects/1\"}` (specific project)"
    )]
    async fn get_project_resources(
        &self,

        #[tool(param)]
        #[schemars(
            description = "Optional project URL to fetch resources from. If not provided, fetches all resources from all projects in the profile. Examples: 'https://github.com/users/username/projects/1', 'https://github.com/orgs/orgname/projects/5'"
        )]
        project_url: Option<String>,
        #[tool(param)]
        #[schemars(
            description = "Optional output format for project resources (light/rich, default: rich). Light format provides minimal information, rich format provides comprehensive details."
        )]
        #[schemars(default)]
        output_option: Option<String>,
    ) -> Result<CallToolResult, McpError> {
        let github_client = GitHubClient::new(self.github_token.clone(), None).map_err(|e| {
            McpError::internal_error(format!("Failed to create GitHub client: {}", e), None)
        })?;

        // Parse output format option, defaulting to rich
        let format = if let Some(option_str) = output_option {
            option_str
                .parse::<OutputOption>()
                .unwrap_or(OutputOption::Rich)
        } else {
            OutputOption::Rich
        };

        let mut content_vec = Vec::new();

        if let Some(project_url_str) = project_url {
            // Fetch resources for specific project
            let project_url = ProjectUrl(project_url_str);
            let project_resources =
                functions::project::get_project_resources(&github_client, project_url)
                    .await
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;

            for project_resource in project_resources {
                let formatted = match format {
                    OutputOption::Light => project_resource_body_markdown_with_timezone_light(
                        &project_resource,
                        self.timezone.as_ref(),
                    ),
                    OutputOption::Rich => project_resource_body_markdown_with_timezone(
                        &project_resource,
                        self.timezone.as_ref(),
                    ),
                };
                content_vec.push(Content::text(formatted.0));
            }
        } else {
            // Fetch resources for all projects in the profile
            let profile_name = self.profile_name.clone().unwrap_or_default();
            let projects = functions::profile::list_projects(profile_name.to_string())
                .await
                .map_err(|e| McpError::internal_error(e, None))?;

            if projects.is_empty() {
                return Ok(CallToolResult {
                    content: vec![Content::text(format!(
                        "No projects found in profile '{}'.",
                        profile_name
                    ))],
                    is_error: Some(false),
                });
            }

            let project_resources =
                functions::project::get_multiple_project_resources(&github_client, projects)
                    .await
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;

            for project_resource in project_resources {
                let formatted = match format {
                    OutputOption::Light => project_resource_body_markdown_with_timezone_light(
                        &project_resource,
                        self.timezone.as_ref(),
                    ),
                    OutputOption::Rich => project_resource_body_markdown_with_timezone(
                        &project_resource,
                        self.timezone.as_ref(),
                    ),
                };
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
        #[schemars(
            description = "Issue URLs to fetch. Examples: ['https://github.com/rust-lang/rust/issues/12345', 'https://github.com/tokio-rs/tokio/issues/5678']"
        )]
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
        #[schemars(
            description = "Pull request URLs to fetch. Examples: ['https://github.com/rust-lang/rust/pull/98765', 'https://github.com/tokio-rs/tokio/pull/4321']"
        )]
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
        description = "Get repository details by URLs. Returns detailed repository information formatted as markdown with comprehensive metadata including URL, description, default branch, mentionable users, labels, milestones, releases (with configurable limit), and timestamps."
    )]
    async fn get_repository_details(
        &self,
        #[tool(param)]
        #[schemars(
            description = "Optional specific repository URLs to fetch. If not provided, fetches all repositories from the profile. Examples: ['https://github.com/rust-lang/rust', 'https://github.com/tokio-rs/tokio']"
        )]
        repository_urls: Option<Vec<String>>,
        #[tool(param)]
        #[schemars(
            description = "Optional limit for number of releases to show per repository (default: 10). Examples: 5, 20"
        )]
        #[schemars(default)]
        showing_release_limit: Option<usize>,
        #[tool(param)]
        #[schemars(
            description = "Optional limit for number of milestones to show per repository (default: 10). Examples: 5, 20"
        )]
        #[schemars(default)]
        showing_milestone_limit: Option<usize>,
    ) -> Result<CallToolResult, McpError> {
        let github_client = GitHubClient::new(self.github_token.clone(), None).map_err(|e| {
            McpError::internal_error(format!("Failed to create GitHub client: {}", e), None)
        })?;

        let repository_urls = if let Some(urls) = repository_urls {
            // Use provided URLs
            urls.into_iter()
                .map(crate::types::RepositoryUrl)
                .collect::<Vec<_>>()
        } else {
            // Load repositories from profile
            let profile_name = self.profile_name.clone().unwrap_or_default();
            let repositories = functions::profile::list_repositories(profile_name.to_string())
                .await
                .map_err(|e| McpError::internal_error(e, None))?;

            if repositories.is_empty() {
                return Ok(CallToolResult {
                    content: vec![Content::text(format!(
                        "No repositories found in profile '{}'.",
                        profile_name
                    ))],
                    is_error: Some(false),
                });
            }

            // Convert RepositoryIds to RepositoryUrls
            repositories
                .into_iter()
                .map(|repo_id| {
                    crate::types::RepositoryUrl(format!(
                        "https://github.com/{}/{}",
                        repo_id.owner, repo_id.repository_name
                    ))
                })
                .collect()
        };

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
                self.timezone.as_ref(),
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

    #[tool(
        description = "Get project details by their URLs. Returns detailed project information formatted as markdown with comprehensive metadata including title, description, creation/update dates, project node ID, and other project properties. The project node ID can be used for project updates."
    )]
    async fn get_project_details(
        &self,
        #[tool(param)]
        #[schemars(
            description = "Project URLs to fetch. Examples: ['https://github.com/users/username/projects/1', 'https://github.com/orgs/orgname/projects/5']"
        )]
        project_urls: Vec<String>,
    ) -> Result<CallToolResult, McpError> {
        let github_client = GitHubClient::new(self.github_token.clone(), None).map_err(|e| {
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
            let formatted = crate::formatter::project::project_body_markdown_with_timezone(
                &project,
                self.timezone.as_ref(),
            );
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

    #[tool(
        description = "Search for issues, PRs, and projects across multiple repositories. The 'github_search_query' parameter is optional and defaults to open issues and PRs. When 'repository_urls' is provided, searches in those repositories. Comprehensive search across multiple resource types. Use get_issues_details and get_pull_request_details functions to get more detailed information. Note: Pagination with cursors is currently disabled - results are returned in a single response."
    )]
    async fn search_in_repositories(
        &self,
        #[tool(param)]
        #[schemars(
            description = "Search query text (optional, default: open issues and PRs). Supports GitHub search syntax. Examples: 'is:pr state:open', 'is:issue label:bug', 'authentication error', 'head:feature-branch', 'is:pr author:username', 'is:issue assignee:username', 'created:2024-01-01..2024-12-31'. Note: Any repo:owner/name specifications in the query will be overridden when searching specific repositories."
        )]
        #[schemars(default = "default_search_query")]
        github_search_query: Option<String>,
        #[tool(param)]
        #[schemars(
            description = "Repository URLs to search in (e.g., ['https://github.com/owner/repo1', 'https://github.com/owner/repo2']). To search repositories from the current profile, use list_repositories_in_current_profile to get repository URLs and pass them to this parameter."
        )]
        repository_urls: Vec<String>,
        #[tool(param)]
        #[schemars(
            description = "Result limit per repository (default 30, max 100). Examples: 10, 50"
        )]
        #[schemars(default = "default_search_limit")]
        limit: Option<usize>,
        #[tool(param)]
        #[schemars(
            description = "Optional search cursors by repository for pagination. Each cursor is associated with a specific repository. Example: [{'cursor': 'Y3Vyc29yOjE=', 'repository_id': {'owner': 'rust-lang', 'repository_name': 'rust'}}]"
        )]
        cursors: Option<Vec<SearchCursorByRepository>>,
        #[tool(param)]
        #[schemars(
            description = "Optional output format for search results (light/rich, default: light). Light format provides minimal information (title, status, URL, assignees/author, truncated body up to 100 chars, comment count, linked resources), rich format provides comprehensive details (full body, all comments, timestamps, labels, etc.)."
        )]
        #[schemars(default)]
        output_option: Option<String>,
    ) -> Result<CallToolResult, McpError> {
        let github_client = GitHubClient::new(self.github_token.clone(), None).map_err(|e| {
            McpError::internal_error(format!("Failed to create GitHub client: {}", e), None)
        })?;

        let limit = limit.unwrap_or(DEFAULT_SEARCH_LIMIT);

        // Convert String to OutputOption
        let format = if let Some(option_str) = output_option {
            option_str.parse::<OutputOption>().unwrap_or_default()
        } else {
            OutputOption::default()
        };

        // Convert String to SearchQuery, using default if not provided
        let query_string = github_search_query.unwrap_or_else(|| DEFAULT_SEARCH_QUERY.to_string());
        let query = SearchQuery::new(query_string);

        // Check if repository_urls is empty and return error
        if repository_urls.is_empty() {
            return Err(McpError::invalid_request(
                "repository_urls cannot be empty. Please provide at least one repository URL."
                    .to_string(),
                None,
            ));
        }

        // Search in specific repositories
        let mut repo_ids = Vec::new();
        for repo_url_str in repository_urls {
            let repo_id =
                crate::types::RepositoryId::parse_url(&crate::types::RepositoryUrl(repo_url_str))
                    .map_err(|e| {
                    McpError::internal_error(format!("Invalid repository ID: {}", e), None)
                })?;
            repo_ids.push(repo_id);
        }
        let repository_urls = repo_ids;

        // Search across repositories
        let search_results = functions::search::search_resources(
            &github_client,
            repository_urls,
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

    #[tool(
        description = "List all repositories registered in the current profile. Returns repository IDs and URLs for repositories managed by the profile."
    )]
    async fn list_repositories_in_current_profile(&self) -> Result<CallToolResult, McpError> {
        let profile_name = self.profile_name.clone().unwrap_or_default().to_string();

        let result = functions::profile::list_repositories(profile_name)
            .await
            .map_err(|e| McpError::internal_error(e, None))?;

        let content = Content::text(serde_json::to_string_pretty(&result).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize result: {}", e), None)
        })?);

        Ok(CallToolResult {
            content: vec![content],
            is_error: Some(false),
        })
    }

    #[tool(
        description = "List all projects registered in the current profile. Returns project IDs and URLs for projects managed by the profile."
    )]
    async fn list_projects_in_current_profile(&self) -> Result<CallToolResult, McpError> {
        let profile_name = self.profile_name.clone().unwrap_or_default().to_string();

        let result = functions::profile::list_projects(profile_name)
            .await
            .map_err(|e| McpError::internal_error(e, None))?;

        let content = Content::text(serde_json::to_string_pretty(&result).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize result: {}", e), None)
        })?);

        Ok(CallToolResult {
            content: vec![content],
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
Get all project resources from specified project(s). Returns all project resources as markdown array including title, description, resource counts, and timestamps. Each project resource includes field IDs that can be used for project field updates. This tool fetches all resources without pagination.

Examples:
```json
// Get all project resources from all projects in profile (default: rich format)
{{"name": "get_project_resources", "arguments": {{}}}}

// Get resources from specific project
{{"name": "get_project_resources", "arguments": {{"project_url": "https://github.com/users/username/projects/1"}}}}

// Get resources with light format
{{"name": "get_project_resources", "arguments": {{"output_option": "light"}}}}

// Get resources with rich format (default)
{{"name": "get_project_resources", "arguments": {{"output_option": "rich"}}}}
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

### 4. get_project_details
Get project details by their URLs. Returns detailed project information formatted as markdown with comprehensive metadata including title, description, creation/update dates, project node ID, and other project properties. The project node ID can be used for project updates.

Examples:
```json
// Get specific projects by URLs
{{"name": "get_project_details", "arguments": {{"project_urls": ["https://github.com/users/username/projects/1", "https://github.com/orgs/orgname/projects/5"]}}}}
```

### 5. get_repository_details
Get repository details by URLs. Returns detailed repository information formatted as markdown array with comprehensive metadata including description, statistics, and configuration details. Releases section can be limited using the showing_release_limit parameter.

Examples:
```json
// Get all repository details from profile
{{"name": "get_repository_details", "arguments": {{}}}}

// Get single repository details
{{"name": "get_repository_details", "arguments": {{"repository_urls": ["https://github.com/rust-lang/rust"]}}}}

// Get multiple repository details
{{"name": "get_repository_details", "arguments": {{"repository_urls": ["https://github.com/rust-lang/rust", "https://github.com/tokio-rs/tokio"]}}}}

// Get repository details with custom release limit
{{"name": "get_repository_details", "arguments": {{"repository_urls": ["https://github.com/rust-lang/rust"], "showing_release_limit": 5}}}}
```

### 6. search_in_repositories
Search across multiple repositories for issues, PRs, and projects. Comprehensive search across multiple resource types with support for specific repository targeting and advanced pagination.

Examples:
```json
// Search in specific repositories
{{"name": "search_in_repositories", "arguments": {{"github_search_query": "memory leak", "repository_urls": ["https://github.com/rust-lang/rust", "https://github.com/tokio-rs/tokio"]}}}}

// Search with default query (open issues and PRs)
{{"name": "search_in_repositories", "arguments": {{"repository_urls": ["https://github.com/tokio-rs/tokio"]}}}}

// Search with specific output format
{{"name": "search_in_repositories", "arguments": {{
    "github_search_query": "async await",
    "repository_urls": ["https://github.com/tokio-rs/tokio"],
    "output_option": "light",
    "limit": 20
}}}}

// Search with pagination cursors
{{"name": "search_in_repositories", "arguments": {{
    "github_search_query": "performance",
    "repository_urls": ["https://github.com/rust-lang/rust"],
    "cursors": [{{"repository_id": {{"owner": "rust-lang", "repository_name": "rust"}}, "cursor": "Y3Vyc29yOnYyOpK5"}}]
}}}}
```

### 7. list_repositories_in_current_profile
List all repositories registered in the current profile. Returns repository IDs and URLs for repositories managed by the profile.

Examples:
```json
// List all repositories in current profile
{{"name": "list_repositories_in_current_profile", "arguments": {{}}}}
```

### 8. list_projects_in_current_profile
List all projects registered in the current profile. Returns project IDs and URLs for projects managed by the profile.

Examples:
```json
// List all projects in current profile
{{"name": "list_projects_in_current_profile", "arguments": {{}}}}
```

## Common Workflows

1. **Profile Management**:
   - Use list_repositories_in_current_profile to get all repositories registered in the current profile
   - Use list_projects_in_current_profile to get all projects registered in the current profile

2. **Repository Search**:
   - Use search_in_repositories to find issues/PRs by keywords across specific repositories
   - Support for pagination using cursors for large result sets
   - Choose between light and rich output formats

3. **Specific Resource Access**:
   - Use get_issues_details to get detailed issue information with comments
   - Use get_pull_request_details to get detailed pull request information with comments

4. **Project Management**:
   - Use get_project_resources to access project boards and associated resources
   - Fetch from all projects in profile or specific project URLs
   - Choose between light and rich output formats (default: rich)

5. **Output Formatting**:
   - Rich format provides comprehensive details including full comments, timestamps, custom fields
   - Light format provides minimal information for quick overview
   - get_project_resources defaults to rich format for detailed project information
   - search_in_repositories defaults to light format for quick search results
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
