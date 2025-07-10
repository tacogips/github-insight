use crate::{
    github::graphql::timeline::timeline_items_query,
    types::{Owner, PullRequestNumber, RepositoryName, SearchCursor},
};
use serde::{Deserialize, Serialize};

const DEFAULT_LIMIT: u8 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PullRequestQueryLimitSize {
    assignee_limit: u8,
    label_limit: u8,
    comment_limit: u8,
    review_request_limit: u8,
    review_limit: u8,
    event_limit: u8,
}

impl Default for PullRequestQueryLimitSize {
    fn default() -> Self {
        Self {
            assignee_limit: DEFAULT_LIMIT,
            label_limit: DEFAULT_LIMIT,
            comment_limit: DEFAULT_LIMIT,
            review_request_limit: DEFAULT_LIMIT,
            review_limit: DEFAULT_LIMIT,
            event_limit: DEFAULT_LIMIT,
        }
    }
}

pub fn pull_request_query_body(limit_size: PullRequestQueryLimitSize) -> String {
    let PullRequestQueryLimitSize {
        assignee_limit,
        label_limit,
        comment_limit,
        review_request_limit,
        review_limit,
        event_limit,
    } = limit_size;
    format!(
        r#"number
                    title
                    body
                    state
                    createdAt
                    updatedAt
                    baseRefName
                    headRefName
                    mergeable
                    merged
                    mergedAt
                    url
                    author {{
                      login
                    }}
                    assignees(first: {}) {{
                      nodes {{
                        login
                      }}
                    }}
                    reviewRequests(first: {}) {{
                      nodes {{
                        requestedReviewer {{
                          __typename
                          ... on User {{
                            login
                          }}
                          ... on Team {{
                            name
                          }}
                        }}
                      }}
                      totalCount
                    }}
                    labels(first: {}) {{
                      nodes {{
                        name
                      }}
                    }}
                    closedAt
                    commits {{
                      totalCount
                    }}
                    additions
                    deletions
                    changedFiles
                    milestone {{
                      number
                    }}
                    locked
                    isDraft
                    comments(first: {}) {{
                      nodes {{
                        id
                        body
                        createdAt
                        updatedAt
                        url
                        author {{
                          login
                        }}
                      }}
                      totalCount
                    }}
                    reviews(first: {}) {{
                      nodes {{
                        id
                        state
                        body
                        createdAt
                        url
                        author {{
                          login
                        }}
                      }}
                      totalCount
                    }}
                    {}"#,
        assignee_limit,
        review_request_limit,
        label_limit,
        comment_limit,
        review_limit,
        timeline_items_query(event_limit)
    )
}

pub fn multi_pull_query_body(
    index: usize,
    pull_request_number: PullRequestNumber,
    limit_size: PullRequestQueryLimitSize,
) -> String {
    format!(
        r#"
        pr{}: pullRequest(number: {}) {{
            {}
        }}"#,
        index,
        pull_request_number,
        pull_request_query_body(limit_size),
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiplePullRequestVariable {
    pub owner: Owner,
    pub repository_name: RepositoryName,
}

pub fn multi_pull_reqeust_query(
    pull_request_numbers: &[PullRequestNumber],
    limit_size: PullRequestQueryLimitSize,
) -> String {
    let each_pr_queries: Vec<String> = pull_request_numbers
        .iter()
        .enumerate()
        .map(|(idx, pull_request_number)| {
            multi_pull_query_body(idx, *pull_request_number, limit_size)
        })
        .collect();

    format!(
        r#"
             query($owner: String!, $repository_name: String!) {{
                 repository(owner: $owner, name: $repository_name) {{
                     {}
                 }}
             }}"#,
        each_pr_queries.join("\n")
    )
}

pub struct SearchPullRequestVariable {
    pub owner: Owner,
    pub per_page: u32,
    pub cursor: Option<SearchCursor>,
}

pub fn pull_request_search_query(
    limit_size: PullRequestQueryLimitSize,
    with_cursor: bool,
) -> String {
    let inner_query = format!(
        r#"
            nodes {{
                __typename
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
        pull_request_query_body(limit_size)
    );

    if with_cursor {
        format!(
            r#"
        query($query: String!, $per_page: Int!, $cursor: String) {{
            search(query: $query, type: PULL_REQUEST, first: $per_page, after: $cursor) {{
                {}
            }}
        "#,
            inner_query
        )
    } else {
        format!(
            r#"
        query($query: String!, $per_page: Int!) {{
            search(query: $query, type: PULL_REQUEST, first: $per_page) {{
                {}
            }}
        "#,
            inner_query
        )
    }
}
