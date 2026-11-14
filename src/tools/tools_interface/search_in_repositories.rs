use crate::formatter::{
    TimezoneOffset,
    issue::{issue_body_markdown_with_timezone, issue_body_markdown_with_timezone_light},
    pull_request::{
        pull_request_body_markdown_with_timezone, pull_request_body_markdown_with_timezone_light,
    },
};
use crate::github::GitHubClient;
use crate::tools::functions;
use crate::types::{OutputOption, SearchCursorByRepository, SearchQuery};
use anyhow::Result;
use rmcp::{Error as McpError, model::*};
use serde_json;

const DEFAULT_SEARCH_LIMIT: usize = 30;
const DEFAULT_SEARCH_QUERY: &str = "state:open";

/// Search for issues, PRs, and projects across multiple repositories
///
/// Comprehensive search across multiple resource types with support for specific
/// repository targeting and advanced pagination.
pub async fn search_in_repositories(
    github_token: &Option<String>,
    timezone: &Option<TimezoneOffset>,
    github_search_query: Option<String>,
    repository_urls: Vec<String>,
    limit: Option<usize>,
    cursors: Option<Vec<SearchCursorByRepository>>,
    output_option: Option<String>,
) -> Result<CallToolResult, McpError> {
    let github_client = GitHubClient::new(github_token.clone(), None).map_err(|e| {
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
                        issue_body_markdown_with_timezone_light(&issue, timezone.as_ref()).0
                    }
                    OutputOption::Rich => {
                        issue_body_markdown_with_timezone(&issue, timezone.as_ref()).0
                    }
                },
                crate::types::IssueOrPullrequest::PullRequest(pr) => match format {
                    OutputOption::Light => {
                        pull_request_body_markdown_with_timezone_light(&pr, timezone.as_ref()).0
                    }
                    OutputOption::Rich => {
                        pull_request_body_markdown_with_timezone(&pr, timezone.as_ref()).0
                    }
                },
            };
            content_vec.push(Content::text(formatted));
        }
    }

    // Add cursor information as JSON
    if !search_results.cursors.is_empty() {
        let cursors_json = serde_json::to_string_pretty(&search_results.cursors).map_err(|e| {
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
