use anyhow::Result;

use crate::github::GitHubClient;
use crate::services::SearchService;
use crate::types::{RepositoryId, SearchCursorByRepository, SearchQuery, SearchResultWithCursors};

/// Search for issues and pull requests across multiple repositories
pub async fn search_resources(
    github_client: &GitHubClient,
    repos: Vec<RepositoryId>,
    query: SearchQuery,
    per_page: Option<u32>,
    cursors: Option<Vec<SearchCursorByRepository>>,
) -> Result<SearchResultWithCursors> {
    let search_service = SearchService::new(github_client.clone());

    search_service
        .search_resources(repos, query, per_page, cursors)
        .await
}
