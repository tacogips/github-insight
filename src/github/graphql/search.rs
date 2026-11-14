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
pub fn overwrite_repo_if_exists(query: SearchQuery, repository_id: &RepositoryId) -> SearchQuery {
    // Remove existing repo:owner/name patterns from the search query
    let cleaned_query = REPO_PATTERN.replace_all(&query.0, "").trim().to_string();

    let search_query = if cleaned_query.is_empty() {
        format!(
            "repo:{}/{}",
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
