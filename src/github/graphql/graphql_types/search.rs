use serde::{Deserialize, Serialize};

use crate::github::graphql::graphql_types::{IssueNode, PageInfo, PullRequestNode};

/// GraphQL response structures for Search API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub search: SearchConnection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConnection {
    pub nodes: Vec<SearchResult>,
    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "__typename")]
pub enum SearchResult {
    #[serde(rename = "Issue")]
    Issue(IssueNode),
    #[serde(rename = "PullRequest")]
    PullRequest(PullRequestNode),
    #[serde(other)]
    Other,
}
