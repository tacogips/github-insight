use crate::types::{Owner, RepositoryName};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryVariable {
    pub owner: Owner,
    pub repository_name: RepositoryName,
}

pub fn repository_query() -> String {
    r#"
        query($owner: String!, $repository_name: String!) {
            repository(owner: $owner, name: $repository_name) {
                name
                description
                primaryLanguage {
                    name
                }
                createdAt
                updatedAt
                defaultBranchRef {
                    name
                }
                milestones(first: 100, states: [OPEN, CLOSED]) {
                    nodes {
                        number
                        title
                        dueOn
                    }
                }
                labels(first: 100) {
                    nodes {
                        name
                        color
                    }
                }
                owner {
                    login
                }
                mentionableUsers(first: 100) {
                    nodes {
                        login
                        name
                        avatarUrl
                    }
                }
                releases(first: 100, orderBy: {field: CREATED_AT, direction: DESC}) {
                    nodes {
                        name
                        tagName
                        description
                        createdAt
                        publishedAt
                        isPrerelease
                        isDraft
                        author {
                            login
                            name
                        }
                        url
                    }
                }
            }
        }
    "#
    .to_string()
}
