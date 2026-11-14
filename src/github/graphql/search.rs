use std::sync::LazyLock;

use regex::Regex;
use serde::Serialize;

use crate::types::{RepositoryId, SearchQuery};

use super::issue::{IssueQueryLimitSize, issue_query_body};
use super::pull_request::{PullRequestQueryLimitSize, pull_request_query_body};
#[derive(Debug, Clone, Serialize)]
pub struct SearchVariable {
    pub query: String,
    pub per_page: u32,
    pub cursor: Option<String>,
}

pub fn search_query(
    issue_limit_size: IssueQueryLimitSize,
    pull_request_limit_size: PullRequestQueryLimitSize,
    with_cursor: bool,
) -> String {
    let inner_query = format!(
        r#"
            nodes {{
                __typename
                ... on Issue {{
                    {}
                    repository {{
                        owner {{
                            login
                        }}
                        name
                    }}
                }}


                ... on PullRequest {{
                    {}
                    repository {{
                        owner {{
                            login
                        }}
                        name
                    }}
                }}


            }}
            pageInfo {{
                hasNextPage
                endCursor
            }}
        "#,
        issue_query_body(issue_limit_size),
        pull_request_query_body(pull_request_limit_size)
    );

    if with_cursor {
        format!(
            r#"
        query($query: String!, $per_page: Int!, $cursor: String) {{
            search(query: $query, type: ISSUE, first: $per_page, after: $cursor) {{
                {}
            }}
        }}"#,
            inner_query
        )
    } else {
        format!(
            r#"
        query($query: String!, $per_page: Int!) {{
            search(query: $query, type: ISSUE, first: $per_page) {{
                {}
            }}
        }}"#,
            inner_query
        )
    }
}

static REPO_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\brepo:[^\s]+").unwrap());

/// Normalizes a repository search query for GitHub GraphQL API.
///
/// This function ensures that the search query targets the specified repository by:
/// 1. Removing any existing `repo:owner/name` patterns from the query
/// 2. Adding the correct `repo:owner/name` specification for the target repository
/// 3. Automatically adding default qualifiers when the query is empty
///
/// # Arguments
///
/// * `query` - The original search query which may contain repository specifications
/// * `repository_id` - The target repository to search in
///
/// # Returns
///
/// A new `SearchQuery` with the repository specification correctly set
///
/// # Behavior
///
/// ## When query is empty or contains only repo: patterns
///
/// If the cleaned query (after removing repo: patterns) is empty, the function adds
/// default qualifiers `is:issue is:pr` to ensure the GitHub GraphQL search API returns
/// results. This is necessary because GitHub's search API does not reliably return
/// results for queries containing only `repo:owner/name` without additional qualifiers.
///
/// Example:
/// ```
/// # use github_insight::types::{SearchQuery, RepositoryId};
/// # use github_insight::github::graphql::search::normalize_repo_search_query;
/// let query = SearchQuery::new("".to_string());
/// let repo = RepositoryId::new("owner".to_string(), "repo".to_string());
/// let result = normalize_repo_search_query(query, &repo);
/// assert_eq!(result.as_str(), "repo:owner/repo is:issue is:pr");
/// ```
///
/// ## When query contains search terms
///
/// If the query contains actual search terms or qualifiers, they are preserved and
/// combined with the repository specification.
///
/// Example:
/// ```
/// # use github_insight::types::{SearchQuery, RepositoryId};
/// # use github_insight::github::graphql::search::normalize_repo_search_query;
/// let query = SearchQuery::new("state:open label:bug".to_string());
/// let repo = RepositoryId::new("owner".to_string(), "repo".to_string());
/// let result = normalize_repo_search_query(query, &repo);
/// assert_eq!(result.as_str(), "repo:owner/repo state:open label:bug");
/// ```
///
/// ## Repository specification replacement
///
/// Any existing `repo:` specifications in the query are removed and replaced with
/// the target repository specification. This ensures searches always target the
/// intended repository regardless of what repo: patterns were in the original query.
///
/// Example:
/// ```
/// # use github_insight::types::{SearchQuery, RepositoryId};
/// # use github_insight::github::graphql::search::normalize_repo_search_query;
/// let query = SearchQuery::new("repo:old/repo some terms".to_string());
/// let repo = RepositoryId::new("new".to_string(), "repo".to_string());
/// let result = normalize_repo_search_query(query, &repo);
/// assert_eq!(result.as_str(), "repo:new/repo some terms");
/// ```
///
/// # Note
///
/// This function is specifically designed for GitHub's GraphQL search API behavior.
/// The automatic addition of `is:issue is:pr` qualifiers ensures consistent search
/// results across different query patterns.
pub fn normalize_repo_search_query(
    query: SearchQuery,
    repository_id: &RepositoryId,
) -> SearchQuery {
    // Remove existing repo:owner/name patterns from the search query
    let cleaned_query = REPO_PATTERN.replace_all(&query.0, "").trim().to_string();

    let search_query = if cleaned_query.is_empty() {
        // When no qualifiers are provided, add "is:issue is:pr" to ensure results are returned
        // GitHub GraphQL search API requires at least one qualifier beyond repo: for reliable results
        format!(
            "repo:{}/{} is:issue is:pr",
            repository_id.owner, repository_id.repository_name
        )
    } else {
        format!(
            "repo:{}/{} {}",
            repository_id.owner, repository_id.repository_name, cleaned_query
        )
    };
    SearchQuery(search_query)
}
