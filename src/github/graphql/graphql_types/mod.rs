mod comment;
pub mod issue;
pub mod pager;
pub mod project;
pub mod pull_request;
pub mod repository;
mod search;
mod timeline;
mod user;

use serde::{Deserialize, Serialize};

pub use comment::*;
pub use issue::*;
pub use pager::*;
pub use project::*;
pub use pull_request::*;
pub use repository::*;
pub use search::*;
pub use timeline::*;
pub use user::*;

#[derive(Debug, Clone, Serialize)]
pub struct GraphQLQuery(pub String);

#[derive(Debug, Clone, Serialize)]
pub struct GraphQLPayload<T: serde::Serialize> {
    pub query: GraphQLQuery,
    pub variables: Option<T>,
}

/// GraphQL response structures for GitHub Projects API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLError {
    pub message: String,
    #[serde(default)]
    pub locations: Vec<serde_json::Value>,
    #[serde(default)]
    pub path: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelsConnection {
    pub nodes: Vec<LabelNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelNode {
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneNode {
    pub number: i32,
}
