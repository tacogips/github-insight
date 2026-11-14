use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

use crate::types::{IssueOrPullrequestId, User, repository::RepositoryId};

use super::label::Label;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PullRequestUrl(pub String);

impl std::fmt::Display for PullRequestUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

static PR_URL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:https?://)?github\.com/([^/]+)/([^/]+)/pull/(\d+)")
        .expect("Failed to compile PR URL regex")
});

/// Wrapper type for pull request numbers providing type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PullRequestNumber(pub u32);

impl PullRequestNumber {
    /// Create a new pull request number
    pub fn new(number: u32) -> Self {
        Self(number)
    }

    /// Get the inner value
    pub fn value(&self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for PullRequestNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Wrapper type for comment numbers providing type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PullRequestCommentNumber(pub u64);

impl PullRequestCommentNumber {
    /// Create a new comment number
    pub fn new(number: u64) -> Self {
        Self(number)
    }

    /// Get the inner value
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for PullRequestCommentNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents the state of a GitHub pull request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumString, Display)]
#[strum(serialize_all = "UPPERCASE")] // For GraphQL API compatibility
pub enum PullRequestState {
    /// Pull request is open
    #[strum(serialize = "OPEN")]
    Open,
    /// Pull request is closed without merging
    #[strum(serialize = "CLOSED")]
    Closed,
    /// Pull request is merged
    #[strum(serialize = "MERGED")]
    Merged,
}

/// Strong-typed pull request identifier with URL parsing capabilities.
///
/// This struct encapsulates all pull request identification logic and URL parsing
/// specific to pull requests. Following domain-driven design, all PR URL
/// parsing and reference extraction logic is self-contained within this domain.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PullRequestId {
    pub git_repository: RepositoryId,
    pub number: u32,
}

impl PullRequestId {
    /// Create new pull request identifier
    pub fn new(repository_id: RepositoryId, number: u32) -> Self {
        Self {
            git_repository: repository_id,
            number,
        }
    }

    /// Returns the pull request URL
    pub fn url(&self) -> String {
        format!("{}/pull/{}", self.git_repository.url(), self.number)
    }

    /// Parse pull request URL to extract repository and PR number
    ///
    /// Domain-specific URL parsing moved from utils to pull request domain.
    pub fn parse_url(url: &PullRequestUrl) -> Result<PullRequestId, String> {
        let url = url.0.to_string();
        let url = url.trim_end_matches('/');

        // Parse GitHub pull request URL patterns:
        // https://github.com/owner/repo/pull/123
        // github.com/owner/repo/pull/123
        if let Some(captures) = PR_URL_REGEX.captures(url) {
            let owner = captures.get(1).unwrap().as_str().to_string();
            let repo = captures.get(2).unwrap().as_str().to_string();
            let number = captures
                .get(3)
                .unwrap()
                .as_str()
                .parse::<u32>()
                .map_err(|_| "Invalid pull request number")?;

            let repository_id = RepositoryId::new(owner, repo);

            return Ok(PullRequestId::new(repository_id, number));
        }

        Err(format!("Invalid GitHub pull request URL format: {}", url))
    }
}

impl std::fmt::Display for PullRequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url())
    }
}

/// Git pull request with full metadata and relationships.
///
/// Contains comprehensive pull request information including reviews, comments,
/// branch information, and cross-references to other resources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub pull_request_id: PullRequestId,
    pub title: String,
    pub body: Option<String>,
    pub state: PullRequestState,
    pub author: Option<User>,
    pub assignees: Vec<User>,
    pub requested_reviewers: Vec<User>,
    pub reviewers: Vec<User>,
    pub labels: Vec<Label>,
    pub head_branch: String,
    pub base_branch: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub merged_at: Option<DateTime<Utc>>,
    pub commits_count: u32,
    pub additions: u32,
    pub deletions: u32,
    pub changed_files: u32,
    pub comments: Vec<PullRequestComment>,
    pub milestone_id: Option<u64>,
    pub draft: bool,
    pub mergeable: Option<bool>,
    pub linked_resources: Vec<IssueOrPullrequestId>,
}

/// A comment ID specific to pull request comments
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GitPullRequestCommentId {
    pub pull_request_id: PullRequestId,
    pub comment_number: PullRequestCommentNumber,
}

impl GitPullRequestCommentId {
    /// Create new pull request comment identifier
    pub fn new(pull_request_id: PullRequestId, comment_number: PullRequestCommentNumber) -> Self {
        Self {
            pull_request_id,
            comment_number,
        }
    }

    /// Returns the comment URL
    pub fn url(&self) -> String {
        format!(
            "{}/pull/{}#issuecomment-{}",
            self.pull_request_id.git_repository.url(),
            self.pull_request_id.number,
            self.comment_number
        )
    }
}

/// Represents a comment on a GitHub pull request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestComment {
    pub comment_number: u64,
    pub body: String,
    pub author: Option<User>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PullRequestComment {
    /// Create new pull request comment
    pub fn new(
        comment_number: u64,
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
