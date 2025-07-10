use crate::github::graphql::graphql_types::project::ProjectNode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserNode {
    pub project_v2: Option<ProjectNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub login: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssigneesConnection {
    pub nodes: Vec<AssigneeNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssigneeNode {
    pub login: String,
}
