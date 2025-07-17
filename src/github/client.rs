use crate::github::error::ApiRetryableError;
use crate::types::{SearchCursor, SearchQuery, SearchResult, SearchResultPager};

use super::graphql::error::classify_graphql_error;
use super::graphql::graphql_types::{GraphQLPayload, GraphQLResponse};
use crate::github::graphql::graphql_types::GraphQLQuery;
use crate::github::graphql::graphql_types::issue::MultipleIssuesResponse;
use crate::github::graphql::graphql_types::project::ProjectResourcesResponse;
use crate::github::graphql::graphql_types::pull_request::MultiplePullRequestsResponse;
use crate::github::graphql::graphql_types::repository::RepositoryResponse;
use crate::github::graphql::issue::{
    IssueQueryLimitSize, MultipleIssueVariable, multi_issue_query,
};
use crate::github::graphql::project::query::{
    ProjectVariable, single_project_query, user_project_query,
};
use crate::github::graphql::pull_request::query::PullRequestQueryLimitSize;
use crate::github::graphql::pull_request::query::{
    MultiplePullRequestVariable, multi_pull_reqeust_query,
};
use crate::github::graphql::repository::query::{RepositoryVariable, repository_query};
use crate::github::graphql::search::overwrite_repo_if_exists;
use crate::github::graphql::search::{SearchVariable, search_query};
use crate::types::ProjectResource;

use anyhow::{Context, Result};
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};
use tokio::time::Duration;

use tokio::time::sleep;
use tracing::{error, info, warn};

/// Default maximum number of retry attempts for API operations
pub const DEFAULT_MAX_RETRY_COUNT: u32 = 15;

/// Maximum number of pull requests to fetch in a single chunk
pub const PULL_REQUEST_CHUNK_SIZE: usize = 30;

const DEFAULT_SEARCH_RESULT_PER_PAGE: u32 = 30;

pub trait GraphQLExecutor {
    #[allow(async_fn_in_trait)]
    async fn execute_graphql<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        query_name: &str,
        payload: GraphQLPayload<T>,
    ) -> Result<GraphQLResponse<R>>;
}

#[derive(Clone)]
pub struct GitHubClient {
    pub(crate) client: octocrab::Octocrab,
}

impl GitHubClient {
    pub fn new(token: Option<String>, timeout: Option<Duration>) -> Result<Self> {
        let mut builder = Octocrab::builder();

        if let Some(token) = token {
            builder = builder.personal_token(token);
        }

        let timeout_duration = timeout.unwrap_or_else(|| Duration::from_secs(10));
        let connection_timeout = if timeout_duration < Duration::from_secs(10) {
            std::cmp::max(timeout_duration, Duration::from_secs(1))
        } else {
            Duration::from_secs(30)
        };

        let read_write_timeout = std::cmp::max(timeout_duration, Duration::from_secs(1));

        builder = builder
            .set_connect_timeout(Some(connection_timeout))
            .set_read_timeout(Some(read_write_timeout))
            .set_write_timeout(Some(read_write_timeout));

        let client = builder.build().context("Failed to build GitHub client")?;

        Ok(Self { client })
    }

    /// Searches for issues and pull requests using GitHub's Search API via GraphQL.
    ///
    /// This method performs a unified search across both issues and pull requests within
    /// a specified repository, returning results as a mixed collection of resources.
    ///
    /// # Arguments
    ///
    /// * `repository_id` - The target repository identifier containing owner and repository name
    /// * `query` - Search query string that follows GitHub's search syntax
    /// * `per_page` - Optional number of results per page (default: 5, GitHub API maximum: 100)
    /// * `cursor` - Optional cursor for pagination to fetch subsequent pages
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `SearchResult` struct that includes:
    /// - `issue_or_pull_requests`: Vector of `IssueOrPullrequest` enum variants representing the search results
    /// - `next_pager`: Optional pagination information for retrieving subsequent pages
    ///
    /// # Errors
    ///
    /// This method can return errors in the following cases:
    /// - GraphQL API request failures (network issues, authentication problems)
    /// - Invalid or malformed search queries
    /// - Repository access permission issues
    /// - Rate limiting by GitHub API
    /// - JSON parsing errors when converting GraphQL response to domain objects
    /// - Timeout errors if the request takes longer than 30 seconds
    ///
    /// # Examples
    ///
    /// ```rust
    /// use github_insight::github::client::GitHubClient;
    /// use github_insight::types::{RepositoryId, SearchQuery, SearchCursor};
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = GitHubClient::new(Some("token".to_string()), None)?;
    /// let repo_id = RepositoryId::new("owner".to_string(), "repo".to_string());
    /// let query = SearchQuery::new("is:open label:bug");
    ///
    /// // Search for open issues with bug label
    /// let search_result = client.search_resources(repo_id.clone(), query.clone(), Some(10), None).await?;
    ///
    /// for result in search_result.issue_or_pull_requests {
    ///     match result {
    ///         github_insight::types::IssueOrPullrequest::Issue(issue) => {
    ///             println!("Found issue: {}", issue.title);
    ///         }
    ///         github_insight::types::IssueOrPullrequest::PullRequest(pr) => {
    ///             println!("Found PR: {}", pr.title);
    ///         }
    ///     }
    /// }
    ///
    /// // Handle pagination
    /// if let Some(pager) = search_result.next_pager {
    ///     if pager.has_next_page {
    ///         if let Some(cursor) = pager.next_page_cursor {
    ///             // Fetch next page
    ///             let next_results = client.search_resources(repo_id, query, Some(10), Some(cursor)).await?;
    ///         }
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Implementation Details
    ///
    /// - Uses GitHub's GraphQL API for efficient searching
    /// - Automatically adds repository scope to the search query if not present
    /// - Supports pagination through cursor-based navigation
    /// - Implements retry logic with exponential backoff for transient failures
    /// - Converts GraphQL response nodes to strongly-typed domain objects
    /// - Filters out unsupported search result types (only issues and PRs are processed)
    ///
    /// # GitHub Search Query Syntax
    ///
    /// The search query follows GitHub's search syntax:
    /// - `is:issue` or `is:pr` - Filter by resource type
    /// - `is:open` or `is:closed` - Filter by state
    /// - `label:bug` - Filter by label
    /// - `author:username` - Filter by author
    /// - `assignee:username` - Filter by assignee
    /// - `created:>2024-01-01` - Filter by creation date
    /// - Full text search is supported across titles, descriptions, and comments
    ///
    /// If no repository scope is specified in the query, it will be automatically
    /// added using the provided `repository_id`.
    pub async fn search_resources(
        &self,
        repository_id: crate::types::RepositoryId,
        query: SearchQuery,
        per_page: Option<u32>,
        cursor: Option<SearchCursor>,
    ) -> Result<SearchResult> {
        let per_page_value = per_page.unwrap_or(DEFAULT_SEARCH_RESULT_PER_PAGE); //default
        let has_cursor = cursor.is_some();

        let query = overwrite_repo_if_exists(query, &repository_id);

        let graphql_query = search_query(
            IssueQueryLimitSize::default(),
            PullRequestQueryLimitSize::default(),
            has_cursor,
        );

        let variables = SearchVariable {
            query: query.as_str().to_string(),
            per_page: per_page_value,
            cursor: cursor.as_ref().map(|c| c.0.clone()),
        };

        let payload = GraphQLPayload {
            query: GraphQLQuery(graphql_query),
            variables: Some(variables),
        };

        // Execute GraphQL query
        let response: crate::github::graphql::graphql_types::GraphQLResponse<
            crate::github::graphql::graphql_types::SearchResponse,
        > = self.execute_graphql("issue_search", payload).await?;

        // Handle response and extract data
        let data = response
            .data
            .ok_or_else(|| anyhow::anyhow!("No data in GraphQL issue search response"))?;

        let mut results = Vec::new();

        // Convert GraphQL response to domain objects
        for search_result in data.search.nodes {
            match search_result {
                crate::github::graphql::graphql_types::SearchResult::Issue(issue_node) => {
                    match crate::types::Issue::try_from(issue_node) {
                        Ok(issue) => results.push(crate::types::IssueOrPullrequest::Issue(issue)),
                        Err(e) => {
                            warn!("Failed to convert search result issue: {}", e);
                            return Err(e);
                        }
                    }
                }
                crate::github::graphql::graphql_types::SearchResult::PullRequest(pr_node) => {
                    match crate::types::PullRequest::try_from((pr_node, repository_id.clone())) {
                        Ok(pull_request) => results
                            .push(crate::types::IssueOrPullrequest::PullRequest(pull_request)),
                        Err(e) => {
                            warn!("Failed to convert search result pull request: {}", e);
                            return Err(e);
                        }
                    }
                }
                _ => {
                    // Skip other result types
                    continue;
                }
            }
        }

        // Create pagination information
        let next_pager = if data.search.page_info.has_next_page {
            Some(data.search.page_info.into())
        } else {
            None
        };

        Ok(crate::types::SearchResult {
            repository_id,
            issue_or_pull_requests: results,
            next_pager,
        })
    }

    /// Fetches multiple pull requests by their numbers
    pub async fn fetch_multiple_pull_requests_by_numbers(
        &self,
        repository_id: crate::types::RepositoryId,
        pr_numbers: &[crate::types::PullRequestNumber],
        limit_size: Option<crate::github::graphql::pull_request::PullRequestQueryLimitSize>,
    ) -> Result<Vec<crate::types::PullRequest>> {
        if pr_numbers.is_empty() {
            return Ok(Vec::new());
        }

        let mut all_pull_requests = Vec::new();

        // Process pull requests in chunks to avoid API limits
        for chunk in pr_numbers.chunks(PULL_REQUEST_CHUNK_SIZE) {
            let chunk_result = self
                .fetch_pull_request_chunk(repository_id.clone(), chunk, limit_size)
                .await?;
            all_pull_requests.extend(chunk_result);
        }

        Ok(all_pull_requests)
    }

    /// Fetches a single chunk of pull requests
    async fn fetch_pull_request_chunk(
        &self,
        repository_id: crate::types::RepositoryId,
        pr_numbers: &[crate::types::PullRequestNumber],
        limit_size: Option<crate::github::graphql::pull_request::PullRequestQueryLimitSize>,
    ) -> Result<Vec<crate::types::PullRequest>> {
        let query = multi_pull_reqeust_query(pr_numbers, limit_size.unwrap_or_default());
        let variables = MultiplePullRequestVariable {
            owner: repository_id.owner.clone(),
            repository_name: repository_id.repository_name.clone(),
        };

        let payload = GraphQLPayload {
            query: GraphQLQuery(query),
            variables: Some(variables),
        };

        // Execute GraphQL query
        let response: crate::github::graphql::graphql_types::GraphQLResponse<
            MultiplePullRequestsResponse,
        > = self.execute_graphql("multi_pull_requests", payload).await?;

        // Handle response and extract data
        let data = response
            .data
            .ok_or_else(|| anyhow::anyhow!("No data in GraphQL multiple_pullrequest response"))?;

        let mut chunk_pull_requests = Vec::new();
        // Convert GraphQL response to domain objects
        for (pr_key, maybe_pr_node) in data.repository.pull_requests {
            if let Some(pr_node) = maybe_pr_node {
                match crate::types::PullRequest::try_from((pr_node, repository_id.clone())) {
                    Ok(pull_request) => chunk_pull_requests.push(pull_request),
                    Err(e) => {
                        warn!("Failed to convert pull request {}: {}", pr_key, e);
                        return Err(e);
                    }
                }
            } else {
                warn!("Pull request {} not found or inaccessible", pr_key);
            }
        }

        Ok(chunk_pull_requests)
    }

    /// Fetches multiple issues by their numbers
    pub async fn fetch_multiple_issues_by_numbers(
        &self,
        repository_id: crate::types::RepositoryId,
        issue_numbers: &[crate::types::IssueNumber],
    ) -> Result<Vec<crate::types::Issue>> {
        if issue_numbers.is_empty() {
            return Ok(Vec::new());
        }

        let query = multi_issue_query(issue_numbers, IssueQueryLimitSize::default());
        let variables = MultipleIssueVariable {
            owner: repository_id.owner.clone(),
            repository_name: repository_id.repository_name.clone(),
        };

        let payload = GraphQLPayload {
            query: GraphQLQuery(query),
            variables: Some(variables),
        };

        // Execute GraphQL query
        let response: crate::github::graphql::graphql_types::GraphQLResponse<
            MultipleIssuesResponse,
        > = self.execute_graphql("multi_issues", payload).await?;

        // Handle response and extract data
        let data = response
            .data
            .ok_or_else(|| anyhow::anyhow!("No data in GraphQL multiple_issues response"))?;

        let mut all_issues = Vec::new();
        // Convert GraphQL response to domain objects
        for (issue_key, maybe_issue_node) in data.repository.issues {
            if let Some(issue_node) = maybe_issue_node {
                match crate::types::Issue::try_from(issue_node) {
                    Ok(issue) => all_issues.push(issue),
                    Err(e) => {
                        warn!("Failed to convert issue {}: {}", issue_key, e);
                        return Err(e);
                    }
                }
            } else {
                warn!("Issue {} not found or inaccessible", issue_key);
            }
        }

        Ok(all_issues)
    }

    /// Convert a project node to a vector of project resources
    async fn convert_project_to_resources(
        &self,
        project: crate::github::graphql::graphql_types::project::ProjectNode,
    ) -> Result<(
        Vec<crate::types::ProjectResource>,
        Option<SearchResultPager>,
    )> {
        let mut resources = Vec::new();
        let mut pager = None;

        if let Some(items) = project.items {
            for item in items.nodes {
                match ProjectResource::try_from(item.clone()) {
                    Ok(resource) => resources.push(resource),
                    Err(e) => {
                        warn!(
                            "Failed to convert project item to resource: {}. Item ID: {}, Content: {:?}",
                            e, item.id, item.content
                        );
                        // Continue processing other items instead of failing the entire operation
                    }
                }
            }

            // Extract pagination information from items connection
            if let Some(page_info) = items.page_info {
                if page_info.has_next_page {
                    pager = Some(page_info.into());
                }
            }
        }

        Ok((resources, pager))
    }

    /// Try to fetch project resources using user project query
    async fn try_user_project_query(
        &self,
        project_id: &crate::types::ProjectId,
        cursor: Option<SearchCursor>,
    ) -> Result<(
        Vec<crate::types::ProjectResource>,
        Option<SearchResultPager>,
    )> {
        let user_start = std::time::Instant::now();
        let user_query = user_project_query(project_id.project_number(), cursor);
        let variables = ProjectVariable {
            owner: project_id.owner().clone(),
        };

        let payload = GraphQLPayload {
            query: GraphQLQuery(user_query),
            variables: Some(variables),
        };

        let response: GraphQLResponse<ProjectResourcesResponse> =
            self.execute_graphql("project_resources", payload).await?;

        info!("User project query took: {:?}", user_start.elapsed());

        if let Some(data) = response.data {
            if let Some(user) = data.user {
                if let Some(project) = user.project_v2 {
                    return self.convert_project_to_resources(project).await;
                }
            }
        }

        Err(anyhow::anyhow!("User project not found: {}", project_id))
    }

    /// Try to fetch project resources using organization project query
    async fn try_organization_project_query(
        &self,
        project_id: &crate::types::ProjectId,
        cursor: Option<SearchCursor>,
    ) -> Result<(
        Vec<crate::types::ProjectResource>,
        Option<SearchResultPager>,
    )> {
        let org_start = std::time::Instant::now();
        let org_query = single_project_query(project_id.project_number(), cursor);
        let variables = ProjectVariable {
            owner: project_id.owner().clone(),
        };

        let payload = GraphQLPayload {
            query: GraphQLQuery(org_query),
            variables: Some(variables),
        };

        let response: GraphQLResponse<ProjectResourcesResponse> =
            self.execute_graphql("project_resources", payload).await?;

        info!("Organization project query took: {:?}", org_start.elapsed());

        if let Some(data) = response.data {
            if let Some(org) = data.organization {
                if let Some(project) = org.project_v2 {
                    return self.convert_project_to_resources(project).await;
                }
            }
        }

        Err(anyhow::anyhow!(
            "Organization project not found: {}",
            project_id
        ))
    }

    /// Iteratively fetch all pages of project resources using pagination
    async fn fetch_all_project_resources_with_pager(
        &self,
        project_id: &crate::types::ProjectId,
        is_user_project: bool,
    ) -> Result<Vec<crate::types::ProjectResource>> {
        let mut all_resources = Vec::new();
        let mut current_cursor = None;

        loop {
            let (resources, pager) = if is_user_project {
                self.try_user_project_query(project_id, current_cursor)
                    .await?
            } else {
                self.try_organization_project_query(project_id, current_cursor)
                    .await?
            };

            // Add current page resources to accumulated results
            all_resources.extend(resources);

            // Check if there's a next page
            if let Some(pager) = pager {
                if pager.has_next_page {
                    if let Some(next_cursor) = pager.next_page_cursor {
                        info!("Fetching next page for project {} with cursor", project_id);
                        current_cursor = Some(next_cursor);
                        continue;
                    }
                }
            }

            // No more pages, break the loop
            break;
        }

        Ok(all_resources)
    }

    pub async fn fetch_all_project_resources(
        &self,
        project_id: crate::types::ProjectId,
    ) -> Result<Vec<crate::types::ProjectResource>> {
        let start_time = std::time::Instant::now();
        info!(
            "Starting fetch_all_project_resources for project {}",
            project_id
        );

        // Use project type to determine which query to try first
        let all_resources = match project_id.project_type() {
            crate::types::ProjectType::User => {
                // Try user project first for user projects
                match self
                    .fetch_all_project_resources_with_pager(&project_id, true)
                    .await
                {
                    Ok(resources) => resources,
                    Err(_) => {
                        // Fallback to organization query if user query fails
                        self.fetch_all_project_resources_with_pager(&project_id, false)
                            .await?
                    }
                }
            }
            crate::types::ProjectType::Organization => {
                // Try organization project first for organization projects
                match self
                    .fetch_all_project_resources_with_pager(&project_id, false)
                    .await
                {
                    Ok(resources) => resources,
                    Err(_) => {
                        // Fallback to user query if organization query fails
                        self.fetch_all_project_resources_with_pager(&project_id, true)
                            .await?
                    }
                }
            }
        };

        info!(
            "Total fetch_all_project_resources took: {:?}, fetched {} resources",
            start_time.elapsed(),
            all_resources.len()
        );

        Ok(all_resources)
    }

    /// Fetches a single project by its identifier
    ///
    /// This method retrieves comprehensive project information including metadata,
    /// title, description, and creation/update timestamps using GitHub's GraphQL API.
    ///
    /// # Arguments
    ///
    /// * `project_id` - The project identifier containing owner, project number, and project type
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `Project` with complete project information
    /// including title, description, creation/update timestamps, and basic metadata.
    ///
    /// # Errors
    ///
    /// This method can return errors in the following cases:
    /// - GraphQL API request failures (network issues, authentication problems)
    /// - Project not found or access permission issues
    /// - Rate limiting by GitHub API
    /// - JSON parsing errors when converting GraphQL response to domain objects
    /// - Timeout errors if the request takes longer than configured timeout
    ///
    /// # Examples
    ///
    /// ```rust
    /// use github_insight::github::client::GitHubClient;
    /// use github_insight::types::{ProjectId, ProjectNumber, ProjectType, Owner};
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = GitHubClient::new(Some("token".to_string()), None)?;
    /// let project_id = ProjectId::new(
    ///     Owner::from("owner".to_string()),
    ///     ProjectNumber::new(1),
    ///     ProjectType::User
    /// );
    ///
    /// // Fetch project information
    /// let project = client.fetch_project(project_id).await?;
    ///
    /// println!("Project: {}", project.title);
    /// println!("Created: {}", project.created_at);
    /// println!("Updated: {}", project.updated_at);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn fetch_project(
        &self,
        project_id: crate::types::ProjectId,
    ) -> Result<crate::types::Project> {
        let start_time = std::time::Instant::now();
        info!("Starting fetch_project for project {}", project_id);

        // Use project type to determine which query to try first
        let project_node = match project_id.project_type() {
            crate::types::ProjectType::User => {
                // Try user project first for user projects
                match self.try_user_project_query_simple(&project_id).await {
                    Ok(project_node) => project_node,
                    Err(_) => {
                        // Fallback to organization query if user query fails
                        self.try_organization_project_query_simple(&project_id)
                            .await?
                    }
                }
            }
            crate::types::ProjectType::Organization => {
                // Try organization project first for organization projects
                match self
                    .try_organization_project_query_simple(&project_id)
                    .await
                {
                    Ok(project_node) => project_node,
                    Err(_) => {
                        // Fallback to user query if organization query fails
                        self.try_user_project_query_simple(&project_id).await?
                    }
                }
            }
        };

        // Convert GraphQL response to domain object
        let project = project_node
            .to_project(project_id.clone())
            .context(format!("Failed to convert project: {}", project_id))?;

        info!("Project fetch completed in {:?}", start_time.elapsed());

        Ok(project)
    }

    /// Try to fetch project using user project query (simple version without pagination)
    async fn try_user_project_query_simple(
        &self,
        project_id: &crate::types::ProjectId,
    ) -> Result<crate::github::graphql::graphql_types::project::ProjectNode> {
        let query = user_project_query(project_id.project_number(), None);
        let variables = ProjectVariable {
            owner: project_id.owner().clone(),
        };

        let payload = GraphQLPayload {
            query: GraphQLQuery(query),
            variables: Some(variables),
        };

        let response: GraphQLResponse<ProjectResourcesResponse> =
            self.execute_graphql("project_fetch", payload).await?;

        if let Some(data) = response.data {
            if let Some(user) = data.user {
                if let Some(project) = user.project_v2 {
                    return Ok(project);
                }
            }
        }

        Err(anyhow::anyhow!("User project not found: {}", project_id))
    }

    /// Try to fetch project using organization project query (simple version without pagination)
    async fn try_organization_project_query_simple(
        &self,
        project_id: &crate::types::ProjectId,
    ) -> Result<crate::github::graphql::graphql_types::project::ProjectNode> {
        let query = single_project_query(project_id.project_number(), None);
        let variables = ProjectVariable {
            owner: project_id.owner().clone(),
        };

        let payload = GraphQLPayload {
            query: GraphQLQuery(query),
            variables: Some(variables),
        };

        let response: GraphQLResponse<ProjectResourcesResponse> =
            self.execute_graphql("project_fetch", payload).await?;

        if let Some(data) = response.data {
            if let Some(org) = data.organization {
                if let Some(project) = org.project_v2 {
                    return Ok(project);
                }
            }
        }

        Err(anyhow::anyhow!(
            "Organization project not found: {}",
            project_id
        ))
    }

    /// Fetches a single repository by its identifier
    ///
    /// This method retrieves comprehensive repository information including metadata,
    /// milestones, labels, and other properties using GitHub's GraphQL API.
    ///
    /// # Arguments
    ///
    /// * `repository_id` - The repository identifier containing owner and repository name
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `GithubRepository` with complete repository information
    /// including description, primary language, creation/update timestamps, milestones, labels,
    /// and default branch information.
    ///
    /// # Errors
    ///
    /// This method can return errors in the following cases:
    /// - GraphQL API request failures (network issues, authentication problems)
    /// - Repository not found or access permission issues
    /// - Rate limiting by GitHub API
    /// - JSON parsing errors when converting GraphQL response to domain objects
    /// - Timeout errors if the request takes longer than configured timeout
    ///
    /// # Examples
    ///
    /// ```rust
    /// use github_insight::github::client::GitHubClient;
    /// use github_insight::types::RepositoryId;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = GitHubClient::new(Some("token".to_string()), None)?;
    /// let repo_id = RepositoryId::new("rust-lang".to_string(), "rust".to_string());
    ///
    /// // Fetch repository information
    /// let repository = client.fetch_repository(repo_id).await?;
    ///
    /// println!("Repository: {}", repository.git_repository_id);
    /// println!("Description: {:?}", repository.description);
    /// println!("Language: {:?}", repository.language);
    /// println!("Created: {}", repository.created_at);
    /// println!("Milestones: {}", repository.milestones.len());
    /// println!("Labels: {}", repository.labels.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn fetch_repository(
        &self,
        repository_id: crate::types::RepositoryId,
    ) -> Result<crate::types::GithubRepository> {
        let query = repository_query();
        let variables = RepositoryVariable {
            owner: repository_id.owner().clone(),
            repository_name: repository_id.repo_name().clone(),
        };

        let payload = GraphQLPayload {
            query: GraphQLQuery(query),
            variables: Some(variables),
        };

        // Execute GraphQL query
        let response: crate::github::graphql::graphql_types::GraphQLResponse<RepositoryResponse> =
            self.execute_graphql("fetch_repository", payload).await?;

        // Handle response and extract data
        let data = response
            .data
            .ok_or_else(|| anyhow::anyhow!("No data in GraphQL repository response"))?;

        let repository_node = data
            .repository
            .ok_or_else(|| anyhow::anyhow!("Repository not found: {}", repository_id))?;

        // Convert GraphQL response to domain object
        let repository = crate::types::GithubRepository::try_from(repository_node)
            .context(format!("Failed to convert repository: {}", repository_id))?;

        Ok(repository)
    }
}

impl GraphQLExecutor for GitHubClient {
    async fn execute_graphql<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        query_name: &str,
        payload: GraphQLPayload<T>,
    ) -> Result<GraphQLResponse<R>> {
        // Use retry logic for GraphQL requests (3 retries for faster failure)
        let result = retry_with_backoff(query_name, Some(3), || async {
            info!(
                "Starting GraphQL request with payload: {}",
                serde_json::to_string_pretty(&payload)
                    .unwrap_or_else(|_| "Invalid JSON".to_string())
            );

            let start_time = std::time::Instant::now();

            // Add timeout to prevent indefinite hanging
            let timeout_duration = std::time::Duration::from_secs(10); // 10 secs timeout

            let response: GraphQLResponse<R> =
                tokio::time::timeout(timeout_duration, self.client.graphql(&payload))
                    .await
                    .map_err(|_| {
                        let duration = start_time.elapsed();
                        error!("GraphQL request timed out after {:?}", duration);
                        ApiRetryableError::Retryable(format!(
                            "GraphQL request timed out after {:?}",
                            duration
                        ))
                    })?
                    .map_err(ApiRetryableError::from_octocrab_error)?;

            let duration = start_time.elapsed();
            info!("GraphQL request completed successfully in {:?}", duration);

            // Check for GraphQL errors within the retry loop
            if let Some(errors) = &response.errors {
                if !errors.is_empty() {
                    let error_msg = errors
                        .iter()
                        .map(|e| e.message.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");

                    // Classify GraphQL errors for retry handling
                    let retry_error = classify_graphql_error(&error_msg);

                    return Err(retry_error);
                }
            }

            Ok(response)
        })
        .await?;

        Ok(result)
    }
}

pub(crate) async fn retry_with_backoff<F, Fut, T>(
    operation_name: &str,
    max_retry_count: Option<u32>,
    execute_operation: F,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = std::result::Result<T, ApiRetryableError>>,
{
    let mut attempt = 0;
    let max_retries = max_retry_count.unwrap_or(DEFAULT_MAX_RETRY_COUNT);

    loop {
        match execute_operation().await {
            Ok(result) => {
                tracing::debug!(
                    "Operation {} succeeded on attempt {}",
                    operation_name,
                    attempt + 1
                );
                return Ok(result);
            }
            Err(e) => {
                // Log detailed error information for debugging
                tracing::warn!(
                    "Operation {} failed on attempt {}: {}",
                    operation_name,
                    attempt + 1,
                    e,
                );

                match e {
                    ApiRetryableError::NonRetryable(_) => {
                        tracing::warn!(
                            "Operation {} returned non-retryable error, failing immediately: {}",
                            operation_name,
                            e
                        );
                        return Err(anyhow::anyhow!(e));
                    }
                    ApiRetryableError::RateLimit => {
                        if attempt < max_retries {
                            attempt += 1;
                            let backoff_delay = Duration::from_millis(
                                (1000_u64).saturating_mul(2_u64.saturating_pow(attempt - 1)),
                            );

                            tracing::warn!(
                                "Rate limit hit for {}, attempt {}/{}, backing off for {:?}",
                                operation_name,
                                attempt,
                                max_retries,
                                backoff_delay
                            );

                            sleep(backoff_delay).await;
                            continue;
                        } else {
                            tracing::warn!(
                                "Rate limit retries exhausted for {} after {} attempts",
                                operation_name,
                                attempt + 1
                            );
                            return Err(anyhow::anyhow!(e));
                        }
                    }
                    ApiRetryableError::Retryable(_) => {
                        if attempt < max_retries {
                            attempt += 1;
                            let backoff_delay = Duration::from_millis(
                                (500_u64).saturating_mul(2_u64.saturating_pow(attempt - 1)),
                            );

                            tracing::warn!(
                                "Retryable error for {}, attempt {}/{}, backing off for {:?}",
                                operation_name,
                                attempt,
                                max_retries,
                                backoff_delay
                            );

                            sleep(backoff_delay).await;
                            continue;
                        } else {
                            tracing::warn!(
                                "Retryable error retries exhausted for {} after {} attempts",
                                operation_name,
                                attempt + 1
                            );
                            return Err(anyhow::anyhow!(e));
                        }
                    }
                }
            }
        }
    }
}
