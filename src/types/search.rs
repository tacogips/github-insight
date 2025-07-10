//! Search and profile types for Git resources
//!
//! This module provides types for search operations, results,
//! and profile management in the GitHub Insight system.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{ProjectId, RepositoryId};

/// Represents a search text string.
///
/// Wraps the search text for type safety and future extensibility.
/// Used for full-text search across issue/PR titles, bodies, and comments.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchQuery(pub String);

impl SearchQuery {
    pub fn new<T: Into<String>>(query: T) -> Self {
        Self(query.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::graphql::search::overwrite_repo_if_exists;
    use crate::types::RepositoryId;

    #[test]
    fn test_overwrite_repo_if_exists() {
        let repo_id = RepositoryId::new("newowner".to_string(), "newrepo".to_string());

        // Test with existing repo pattern
        let query = SearchQuery::new("repo:oldowner/oldrepo some search terms".to_string());
        let result = overwrite_repo_if_exists(query, &repo_id);
        assert_eq!(result.as_str(), "repo:newowner/newrepo some search terms");

        // Test with multiple repo patterns
        let query = SearchQuery::new("repo:first/repo some terms repo:second/repo".to_string());
        let result = overwrite_repo_if_exists(query, &repo_id);
        assert_eq!(result.as_str(), "repo:newowner/newrepo some terms");

        // Test without existing repo pattern
        let query = SearchQuery::new("some search terms".to_string());
        let result = overwrite_repo_if_exists(query, &repo_id);
        assert_eq!(result.as_str(), "repo:newowner/newrepo some search terms");

        // Test with empty query
        let query = SearchQuery::new("".to_string());
        let result = overwrite_repo_if_exists(query, &repo_id);
        assert_eq!(result.as_str(), "repo:newowner/newrepo");

        // Test with only repo pattern
        let repo_id = RepositoryId::new("test".to_string(), "test".to_string());
        let query = SearchQuery::new("repo:other/repo".to_string());
        let result = overwrite_repo_if_exists(query, &repo_id);
        assert_eq!(result.as_str(), "repo:test/test");
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchCursor(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchCursorByRepository {
    pub cursor: SearchCursor,
    pub repositor_id: RepositoryId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchResultPager {
    pub next_page_cursor: Option<SearchCursor>,
    pub has_next_page: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultWithCursors {
    pub results: Vec<crate::types::IssueOrPullrequest>,
    pub cursors: Vec<SearchCursorByRepository>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectResourceResultWithCursors {
    pub project_id: ProjectId,
    pub resources: Vec<crate::types::ProjectResource>,
    pub cursor: SearchCursor,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchCursorByProject {
    pub cursor: SearchCursor,
    pub project_id: ProjectId,
}
