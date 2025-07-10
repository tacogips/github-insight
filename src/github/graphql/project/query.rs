use crate::types::{Owner, ProjectNumber, SearchCursor};
use serde::{Deserialize, Serialize};

const DEFAULT_LIMIT: u8 = 100;
const DEFAULT_ITEM_IMIT: u8 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProjectQueryLimitSize {
    item_limit: u8,
    field_limit: u8,
    assignee_limit: u8,
    label_limit: u8,
}

impl Default for ProjectQueryLimitSize {
    fn default() -> Self {
        Self {
            item_limit: DEFAULT_ITEM_IMIT,
            field_limit: DEFAULT_LIMIT,
            assignee_limit: DEFAULT_LIMIT,
            label_limit: DEFAULT_LIMIT,
        }
    }
}

fn project_query_body(limit_size: ProjectQueryLimitSize, cursor: Option<SearchCursor>) -> String {
    let ProjectQueryLimitSize {
        item_limit,
        field_limit,
        assignee_limit,
        label_limit,
    } = limit_size;
    let cursor_param = cursor
        .map(|c| format!(r#", after: "{}""#, c.0))
        .unwrap_or_default();
    format!(
        r#"
                  id
                  title
                  url
                  createdAt
                  updatedAt
                  shortDescription
                  readme
                  public
                  closed
                  closedAt
                  owner {{
                    __typename
                    ... on User {{
                      login
                    }}
                    ... on Organization {{
                      login
                    }}
                  }}
                  items(first: {}{}) {{
                    nodes {{
                      id
                      content {{
                        __typename
                        ... on Issue {{
                          id
                          number
                          title
                          url
                          state
                          createdAt
                          updatedAt
                          author {{
                            __typename
                            ... on User {{
                              login
                            }}
                            ... on Organization {{
                              login
                            }}
                          }}
                          assignees(first: {}) {{
                            nodes {{
                              login
                            }}
                          }}
                          labels(first: {}) {{
                            nodes {{
                              name
                            }}
                          }}
                        }}
                        ... on PullRequest {{
                          id
                          number
                          title
                          url
                          state
                          createdAt
                          updatedAt
                          author {{
                            __typename
                            ... on User {{
                              login
                            }}
                            ... on Organization {{
                              login
                            }}
                          }}
                          assignees(first: {}) {{
                            nodes {{
                              login
                            }}
                          }}
                          labels(first: {}) {{
                            nodes {{
                              name
                            }}
                          }}
                        }}
                        ... on DraftIssue {{
                          id
                          title
                          createdAt
                          updatedAt
                        }}
                      }}
                      fieldValues(first: {}) {{
                        nodes {{
                          __typename
                          ... on ProjectV2ItemFieldTextValue {{
                            field {{
                              ... on ProjectV2FieldCommon {{
                                id
                                name
                              }}
                            }}
                            text
                          }}
                          ... on ProjectV2ItemFieldSingleSelectValue {{
                            field {{
                              ... on ProjectV2FieldCommon {{
                                id
                                name
                              }}
                            }}
                            name
                          }}
                          ... on ProjectV2ItemFieldNumberValue {{
                            field {{
                              ... on ProjectV2FieldCommon {{
                                id
                                name
                              }}
                            }}
                            number
                          }}
                          ... on ProjectV2ItemFieldDateValue {{
                            field {{
                              ... on ProjectV2FieldCommon {{
                                id
                                name
                              }}
                            }}
                            date
                          }}
                        }}
                      }}
                    }}
                    pageInfo {{
                      hasNextPage
                      endCursor
                    }}
                  }}
                "#,
        item_limit,
        cursor_param,
        assignee_limit,
        label_limit,
        assignee_limit,
        label_limit,
        field_limit
    )
}

pub fn single_project_query_body(
    project_number: ProjectNumber,
    limit_size: ProjectQueryLimitSize,
    cursor: Option<SearchCursor>,
) -> String {
    format!(
        r#"
        projectV2(number: {}) {{
            {}
        }} "#,
        project_number.value(),
        project_query_body(limit_size, cursor),
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectVariable {
    pub owner: Owner,
}

pub fn single_project_query(project_number: ProjectNumber, cursor: Option<SearchCursor>) -> String {
    format!(
        r#"
             query($owner: String!) {{
                 organization(login: $owner) {{
                     {}
                 }}
             }}
        "#,
        single_project_query_body(project_number, ProjectQueryLimitSize::default(), cursor)
    )
}

pub fn user_project_query(project_number: ProjectNumber, cursor: Option<SearchCursor>) -> String {
    format!(
        r#"
             query($owner: String!) {{
                 user(login: $owner) {{
                     {}
                 }}
             }}
        "#,
        single_project_query_body(project_number, ProjectQueryLimitSize::default(), cursor)
    )
}

pub fn multi_project_query_body(
    index: usize,
    project_number: ProjectNumber,
    limit_size: ProjectQueryLimitSize,
) -> String {
    format!(
        r#"
        proj{}: projectV2(number: {}) {{
            {}
        }} "#,
        index,
        project_number.value(),
        project_query_body(limit_size, None),
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultipleProjectVariable {
    pub owner: Owner,
}

pub fn multi_project_query(project_numbers: &[ProjectNumber]) -> String {
    let each_project_queries: Vec<String> = project_numbers
        .iter()
        .enumerate()
        .map(|(idx, project_number)| {
            multi_project_query_body(idx, *project_number, ProjectQueryLimitSize::default())
        })
        .collect();

    format!(
        r#"
             query($owner: String!) {{
                 organization(login: $owner) {{
                     {}
                 }}
             }}
        "#,
        each_project_queries.join("\n")
    )
}

pub fn multi_user_project_query(project_numbers: &[ProjectNumber]) -> String {
    let each_project_queries: Vec<String> = project_numbers
        .iter()
        .enumerate()
        .map(|(idx, project_number)| {
            multi_project_query_body(idx, *project_number, ProjectQueryLimitSize::default())
        })
        .collect();

    format!(
        r#"
             query($owner: String!) {{
                 user(login: $owner) {{
                     {}
                 }}
             }}
        "#,
        each_project_queries.join("\n")
    )
}
