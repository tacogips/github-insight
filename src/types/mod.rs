//! Core type system and domain definitions
//!
//! This module provides the central type definitions for the GitHub Insight system,
//! following domain-driven design principles. All types are strongly-typed and
//! provide comprehensive validation and conversion capabilities.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub use crate::github::graphql::graphql_types::repository::MilestoneNumber;

pub mod issue;
pub mod label;
pub mod profile;
pub mod project;
pub mod pull_request;
pub mod repository;
pub mod search;
pub mod user;

pub use issue::*;
pub use profile::*;
pub use project::*;
pub use pull_request::*;
pub use repository::*;
pub use search::*;
pub use user::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum IssueOrPullrequestId {
    IssueId(IssueId),
    PullrequestId(PullRequestId),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueOrPullrequest {
    Issue(Issue),
    PullRequest(PullRequest),
}

pub struct SearchResult {
    pub repository_id: RepositoryId,
    pub issue_or_pull_requests: Vec<crate::types::IssueOrPullrequest>,
    pub next_pager: Option<SearchResultPager>,
}

/// Output format options for search results
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutputOption {
    /// Light format with minimal information
    #[default]
    Light,
    /// Rich format with comprehensive details
    Rich,
}
