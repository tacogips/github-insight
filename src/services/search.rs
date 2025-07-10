use anyhow::Result;

use crate::github::GitHubClient;
use crate::types::{
    RepositoryId, SearchCursorByRepository, SearchQuery, SearchResult, SearchResultWithCursors,
};

/// Service for performing searches across GitHub data.
///
/// Provides various search methods including full-text search,
/// semantic search, and filtering by multiple criteria.
pub struct SearchService {
    github_client: GitHubClient,
}

impl SearchService {
    /// Creates a new search service instance with GitHub client and repository manager
    pub fn new(github_client: GitHubClient) -> Self {
        Self { github_client }
    }

    /// Searches for issues and pull requests across multiple repositories
    pub async fn search_resources(
        &self,
        repos: Vec<RepositoryId>,
        query: SearchQuery,
        per_page: Option<u32>,
        cursors: Option<Vec<SearchCursorByRepository>>,
    ) -> Result<SearchResultWithCursors> {
        use futures::stream::{self, StreamExt};
        use std::collections::HashMap;

        let cursor_map: HashMap<_, _> = cursors
            .as_ref()
            .map(|cursors| {
                cursors
                    .iter()
                    .map(|c| (c.repositor_id.clone(), c.cursor.clone()))
                    .collect()
            })
            .unwrap_or_default();

        // Search across all repositories concurrently
        let search_futures = repos.into_iter().map(|repo_id| {
            let github_client = self.github_client.clone();
            let query = query.clone();
            let cursor = cursor_map.get(&repo_id).cloned();

            async move {
                match github_client
                    .search_resources(repo_id.clone(), query, per_page, cursor)
                    .await
                {
                    Ok(search_result) => Ok(search_result),
                    Err(e) => {
                        tracing::warn!("Failed to search resources in {}: {}", repo_id, e);
                        Err(e)
                    }
                }
            }
        });

        let results: Vec<Result<SearchResult>> = stream::iter(search_futures)
            .buffer_unordered(10) // Process up to 10 repositories concurrently
            .collect()
            .await;

        // Collect all successful results and merge them
        let mut all_results = Vec::new();
        let mut next_cursors = Vec::new();

        for search_result in results.into_iter().flatten() {
            all_results.extend(search_result.issue_or_pull_requests);

            // Track pagination info for each repository
            if let Some(pager) = search_result.next_pager {
                if pager.has_next_page {
                    next_cursors.push(SearchCursorByRepository {
                        cursor: pager
                            .next_page_cursor
                            .unwrap_or_else(|| crate::types::SearchCursor("".to_string())),
                        repositor_id: search_result.repository_id,
                    });
                }
            }
        }

        // Return results with cursors
        let result_with_cursors = SearchResultWithCursors {
            results: all_results,
            cursors: next_cursors,
        };

        Ok(result_with_cursors)
    }
}
