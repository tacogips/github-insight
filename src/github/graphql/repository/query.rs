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
            }
        }
    "#
    .to_string()
}
