use crate::types::{IssueNumber, Owner, RepositoryName};
use serde::{Deserialize, Serialize};

const DEFAULT_LIMIT: u8 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IssueQueryLimitSize {
    assignee_limit: u8,
    label_limit: u8,
    comment_limit: u8,
    event_limit: u8,
}
impl Default for IssueQueryLimitSize {
    fn default() -> Self {
        Self {
            assignee_limit: DEFAULT_LIMIT,
            label_limit: DEFAULT_LIMIT,
            comment_limit: DEFAULT_LIMIT,
            event_limit: DEFAULT_LIMIT,
        }
    }
}

pub fn issue_query_body(limit_size: IssueQueryLimitSize) -> String {
    let IssueQueryLimitSize {
        assignee_limit,
        label_limit,
        comment_limit,
        event_limit,
    } = limit_size;

    format!(
        r#"number
                    title
                    body
                    state
                    createdAt
                    updatedAt
                    closedAt
                    url
                    author {{
                      login
                    }}
                    assignees(first: {}) {{
                      nodes {{
                        login
                      }}
                    }}
                    labels(first: {}) {{
                      nodes {{
                        name
                        color
                      }}
                    }}
                    milestone {{
                      number
                    }}
                    locked
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
                    {}"#,
        assignee_limit,
        label_limit,
        comment_limit,
        crate::github::graphql::timeline::timeline_items_query(event_limit)
    )
}

pub fn issue_connection_query_body(limit_size: IssueQueryLimitSize) -> String {
    format!(
        r#"
           nodes {{
               {}
               repository {{
                   owner {{
                       login
                   }}
                   name
               }}
           }}
           pageInfo {{
               hasNextPage
               endCursor
           }}
       }}"#,
        issue_query_body(limit_size)
    )
}

pub fn multi_issue_query_body(
    index: usize,
    issue_number: IssueNumber,
    limit_size: IssueQueryLimitSize,
) -> String {
    format!(
        r#"
        issue{}: issue(number: {}) {{
            {}
            repository {{
                owner {{
                    login
                }}
                name
            }}
        }}"#,
        index,
        issue_number,
        issue_query_body(limit_size),
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultipleIssueVariable {
    pub owner: Owner,
    pub repository_name: RepositoryName,
}

pub fn multi_issue_query(issue_numbers: &[IssueNumber], limit_size: IssueQueryLimitSize) -> String {
    let each_issue_queries: Vec<String> = issue_numbers
        .iter()
        .enumerate()
        .map(|(idx, issue_number)| multi_issue_query_body(idx, *issue_number, limit_size))
        .collect();

    format!(
        r#"
             query($owner: String!, $repository_name: String!) {{
                 repository(owner: $owner, name: $repository_name) {{
                     {}
                 }}
             }}"#,
        each_issue_queries.join("\n")
    )
}

pub fn issue_search_query(limit_size: IssueQueryLimitSize, with_cursor: bool) -> String {
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
            }}
            pageInfo {{
                hasNextPage
                endCursor
            }}
        "#,
        issue_query_body(limit_size)
    );

    if with_cursor {
        format!(
            r#"
        query($query: String!, $per_page: Int!, $cursor: String) {{
            search(query: $query, type: ISSUE, first: $per_page, after: $cursor) {{
                {}
            }}
        "#,
            inner_query
        )
    } else {
        format!(
            r#"
        query($query: String!, $per_page: Int!) {{
            search(query: $query, type: ISSUE, first: $per_page) {{
                {}
            }}
        "#,
            inner_query
        )
    }
}
