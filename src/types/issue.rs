//! Issue domain types and URL parsing
//!
//! This module contains the Issue domain types with comprehensive URL parsing
//! and cross-reference extraction capabilities. Following domain-driven design
//! principles, all issue-specific URL parsing and reference extraction logic
//! is contained within this module.

use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use crate::types::{User, repository::RepositoryId};

use super::IssueOrPullrequestId;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IssueUrl(pub String);

impl std::fmt::Display for IssueUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

static ISSUE_URL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:https?://)?github\.com/([^/]+)/([^/]+)/issues/(\d+)")
        .expect("Failed to compile issue URL regex")
});

/// Wrapper type for issue numbers providing type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IssueNumber(pub u32);

impl IssueNumber {
    /// Create a new issue number
    pub fn new(number: u32) -> Self {
        Self(number)
    }

    /// Get the inner value
    pub fn value(&self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for IssueNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Wrapper type for comment numbers providing type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommentNumber(pub u32);

impl CommentNumber {
    /// Create a new comment number
    pub fn new(number: u32) -> Self {
        Self(number)
    }

    /// Get the inner value
    pub fn value(&self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for CommentNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents the state of a GitHub issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumString, Display)]
#[strum(serialize_all = "UPPERCASE")] // For GraphQL API compatibility
pub enum IssueState {
    /// Issue is open and active
    #[strum(serialize = "OPEN")]
    Open,
    /// Issue is closed  
    #[strum(serialize = "CLOSED")]
    Closed,
}

/// Strong-typed issue identifier with URL parsing capabilities.
///
/// This struct encapsulates all issue identification logic and URL parsing
/// specific to issues. Following domain-driven design, all issue URL
/// parsing and reference extraction logic is self-contained within this domain.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IssueId {
    pub git_repository: RepositoryId,
    pub number: u32,
}

impl IssueId {
    /// Create new issue identifier
    pub fn new(git_repository: RepositoryId, number: u32) -> Self {
        Self {
            git_repository,
            number,
        }
    }

    /// Returns the issue URL
    pub fn url(&self) -> String {
        format!("{}/issues/{}", self.git_repository.url(), self.number)
    }

    /// Parse issue identifier from GitHub issue URL
    /// - "https://github.com/owner/repo/issues/123" - GitHub issue URL
    pub fn parse_url(input: &IssueUrl) -> Result<Self, String> {
        let input = input.0.to_string();
        let input_str = input.trim_end_matches('/');

        // Handle GitHub issue URLs
        if let Some(captures) = ISSUE_URL_REGEX.captures(input_str) {
            let owner = captures.get(1).unwrap().as_str().to_string();
            let repo = captures.get(2).unwrap().as_str().to_string();
            let number = captures
                .get(3)
                .unwrap()
                .as_str()
                .parse::<u32>()
                .map_err(|e| format!("Invalid issue number: {}", e))?;

            let repository_id = crate::types::repository::RepositoryId::new(owner, repo);
            return Ok(Self::new(repository_id, number));
        }

        Err(format!("Invalid issue URL format: {}", input_str))
    }
}

impl std::fmt::Display for IssueId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url())
    }
}

/// Git issue with full metadata and relationships.
///
/// Contains comprehensive issue information including comments, labels,
/// assignees, and cross-references to other resources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub issue_id: IssueId,
    pub title: String,
    pub body: Option<String>,
    pub state: IssueState,
    pub author: String,
    pub assignees: Vec<String>,
    pub labels: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub comments_count: u32,
    pub comments: Vec<IssueComment>,
    pub milestone_id: Option<u64>,
    pub locked: bool,
    pub linked_resources: Vec<IssueOrPullrequestId>,
}

impl Issue {
    /// Create new issue with complete metadata
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_all_fields(
        issue_id: IssueId,
        title: String,
        body: Option<String>,
        state: IssueState,
        author: String,
        assignees: Vec<String>,
        labels: Vec<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        closed_at: Option<DateTime<Utc>>,
        comments_count: u32,
        comments: Vec<IssueComment>,
        milestone_id: Option<u64>,
        locked: bool,
        linked_resources: Vec<IssueOrPullrequestId>,
    ) -> Self {
        Self {
            issue_id,
            title,
            body,
            state,
            author,
            assignees,
            labels,
            created_at,
            updated_at,
            closed_at,
            comments_count,
            comments,
            milestone_id,
            locked,
            linked_resources,
        }
    }
}

/// A comment ID specific to issue comments
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GitIssueCommentId {
    pub git_issue_id: IssueId,
    pub comment_id: u64,
}

impl GitIssueCommentId {
    /// Create new issue comment identifier
    pub fn new(git_issue_id: IssueId, comment_id: u64) -> Self {
        Self {
            git_issue_id,
            comment_id,
        }
    }

    /// Returns the comment URL
    pub fn url(&self) -> String {
        format!(
            "{}#issuecomment-{}",
            self.git_issue_id.url(),
            self.comment_id
        )
    }
}

impl std::fmt::Display for GitIssueCommentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}#issuecomment-{}", self.git_issue_id, self.comment_id)
    }
}

/// Represents a comment on a GitHub issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueComment {
    pub comment_number: IssueCommentNumber,
    pub body: String,
    pub author: Option<User>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl IssueComment {
    /// Create new issue comment
    pub fn new(
        comment_number: IssueCommentNumber,
        body: String,
        author: Option<User>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            comment_number,
            body,
            author,
            created_at,
            updated_at,
        }
    }
}

/// Wrapper type for comment numbers providing type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IssueCommentNumber(pub u64);

impl IssueCommentNumber {
    /// Create a new comment number
    pub fn new(number: u64) -> Self {
        Self(number)
    }

    /// Get the inner value
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for IssueCommentNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
