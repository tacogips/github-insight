//! Repository domain types and URL parsing
//!
//! This module contains the Repository domain types with comprehensive URL parsing
//! capabilities. Following domain-driven design principles, all repository-specific
//! URL parsing logic is contained within this module, eliminating dependencies on
//! generic utils for domain-specific functionality.

use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{User, label::Label};
use crate::github::graphql::graphql_types::repository::RepositoryNode;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct Branch(pub String);

impl Branch {
    pub fn new<T: Into<String>>(branch: T) -> Self {
        Self(branch.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Repository URL wrapper for type safety
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryUrl(pub String);

static HTTPS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:https?://)?github\.com/([^/]+)/([^/]+?)(?:\.git)?(?:/.*)?/?$")
        .expect("Failed to compile HTTPS regex")
});

static SSH_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"git@github\.com:([^/]+)/([^/]+?)(?:\.git)?/?$")
        .expect("Failed to compile SSH regex")
});

static SIMPLE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^([^/]+)/([^/]+)$").expect("Failed to compile simple regex"));

/// Owner name wrapper for type safety
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, PartialOrd, Ord,
)]
pub struct Owner(pub String);

impl Owner {
    /// Create new owner with validation
    pub fn new(owner: String) -> Self {
        Self(owner)
    }

    /// Get the string value
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert to String
    pub fn into_string(self) -> String {
        self.0
    }
}

impl std::fmt::Display for Owner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Owner {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for Owner {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Repository name wrapper for type safety
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, PartialOrd, Ord,
)]
pub struct RepositoryName(pub String);

impl RepositoryName {
    /// Create new repository name with validation
    pub fn new(repo_name: String) -> Self {
        Self(repo_name)
    }

    /// Get the string value
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert to String
    pub fn into_string(self) -> String {
        self.0
    }
}

impl std::fmt::Display for RepositoryName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for RepositoryName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for RepositoryName {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl RepositoryUrl {
    /// Create new repository URL with validation
    pub fn new(url: String) -> Self {
        Self(url)
    }

    /// Get the string value
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert to String
    pub fn into_string(self) -> String {
        self.0
    }
}

impl std::fmt::Display for RepositoryUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for RepositoryUrl {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for RepositoryUrl {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MilestoneNumber(pub u64);

impl std::fmt::Display for MilestoneNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct MilestoneName(pub String);

impl std::fmt::Display for MilestoneName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Repository milestone association mapping milestone IDs to names.
///
/// This struct represents the relationship between a repository and its milestones,
/// storing both the numeric milestone ID and the human-readable milestone name.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepositoryMilestone {
    pub milestone_number: MilestoneNumber,
    /// The human-readable milestone name as displayed in GitHub
    pub milestone_name: MilestoneName,

    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReleaseId(pub String);

impl std::fmt::Display for ReleaseId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReleaseName(pub String);

impl std::fmt::Display for ReleaseName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TagName(pub String);

impl std::fmt::Display for TagName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Repository release information from GitHub
///
/// This struct represents a GitHub release with all its metadata,
/// including version information, timestamps, and author details.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepositoryRelease {
    /// The release ID (derived from tag name if name is not available)
    pub release_id: ReleaseId,
    /// The human-readable release name
    pub name: ReleaseName,
    /// The git tag name associated with this release
    pub tag_name: TagName,
    /// The release description/body
    pub description: Option<String>,
    /// When the release was created
    pub created_at: DateTime<Utc>,
    /// When the release was published (may be None for drafts)
    pub published_at: Option<DateTime<Utc>>,
    /// Whether this is a pre-release
    pub is_prerelease: bool,
    /// Whether this is a draft
    pub is_draft: bool,
    /// The author of the release
    pub author: Option<User>,
    /// The URL to the release page
    pub url: String,
}

/// A strongly-typed repository identifier for GitHub repositories
///
/// This struct encapsulates all repository identification logic and URL parsing
/// specific to repositories. Following domain-driven design, all repository URL
/// parsing logic is self-contained within this domain.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, PartialOrd, Ord,
)]
pub struct RepositoryId {
    pub owner: Owner,
    pub repository_name: RepositoryName,
}

impl RepositoryId {
    /// Parse repository identifier from various input formats
    /// - "https://github.com/owner/repo" - GitHub URL
    /// - "git@github.com:owner/repo.git" - SSH format
    pub fn parse_url(input: &RepositoryUrl) -> Result<Self, String> {
        let input_str = input.as_str().trim_end_matches('/');

        // Handle GitHub HTTPS URLs
        if let Some(captures) = HTTPS_REGEX.captures(input_str) {
            let owner = captures.get(1).unwrap().as_str().to_string();
            let repo = captures.get(2).unwrap().as_str().to_string();
            return Ok(Self::new(owner, repo));
        }

        // Handle SSH URLs (git@github.com:owner/repo.git)
        if let Some(captures) = SSH_REGEX.captures(input_str) {
            let owner = captures.get(1).unwrap().as_str().to_string();
            let repo = captures.get(2).unwrap().as_str().to_string();
            return Ok(Self::new(owner, repo));
        }

        // Handle simple owner/repo format
        if let Some(captures) = SIMPLE_REGEX.captures(input_str) {
            let owner = captures.get(1).unwrap().as_str().to_string();
            let repo = captures.get(2).unwrap().as_str().to_string();
            return Ok(Self::new(owner, repo));
        }

        Err(format!("Invalid repository format: {}", input_str))
    }

    /// Creates a new repository identifier with validation
    pub fn new<T1: Into<String>, T2: Into<String>>(owner: T1, name: T2) -> Self {
        Self {
            owner: Owner::new(owner.into()),
            repository_name: RepositoryName::new(name.into()),
        }
    }

    /// Check if input string is in provider format (e.g., github:owner/repo)
    /// Returns the owner part of the repository
    pub fn owner(&self) -> &Owner {
        &self.owner
    }

    /// Returns the repository name
    pub fn repo_name(&self) -> &RepositoryName {
        &self.repository_name
    }

    /// Returns the repository URL
    pub fn url(&self) -> String {
        format!("https://github.com/{}/{}", self.owner, self.repository_name)
    }

    /// Returns the short name (repository name only)
    ///TODO delete
    pub fn short_name(&self) -> &str {
        self.repository_name.as_str()
    }

    /// Returns the full name (owner/repository_name format)
    ///TODO delete
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner, self.repository_name)
    }
}

impl std::fmt::Display for RepositoryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url())
    }
}

/// Git repository metadata with comprehensive information
///
/// Contains repository metadata and relationships, including milestones
/// for search filtering support.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubRepository {
    pub git_repository_id: RepositoryId,
    pub description: Option<String>,
    pub language: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub milestones: Vec<RepositoryMilestone>,
    pub default_branch: Option<Branch>,
    pub labels: Vec<Label>,
    pub users: Vec<User>,
    pub releases: Vec<RepositoryRelease>,
}

impl GithubRepository {
    /// Create new repository with basic metadata
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        git_repository_id: RepositoryId,
        description: Option<String>,
        language: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        milestones: Vec<RepositoryMilestone>,
        default_branch: Option<Branch>,
        labels: Vec<Label>,
        users: Vec<User>,
        releases: Vec<RepositoryRelease>,
    ) -> Self {
        Self {
            git_repository_id,
            description,
            language,
            created_at,
            updated_at,
            milestones,
            default_branch,
            labels,
            users,
            releases,
        }
    }

    /// Get repository identifier
    pub fn repository_id(&self) -> RepositoryId {
        self.git_repository_id.clone()
    }
}

impl TryFrom<RepositoryNode> for GithubRepository {
    type Error = anyhow::Error;

    fn try_from(node: RepositoryNode) -> Result<Self, Self::Error> {
        use anyhow::Context;

        // Parse timestamps
        let created_at = chrono::DateTime::parse_from_rfc3339(&node.created_at)
            .context("Failed to parse created_at timestamp")?
            .with_timezone(&Utc);

        let updated_at = chrono::DateTime::parse_from_rfc3339(&node.updated_at)
            .context("Failed to parse updated_at timestamp")?
            .with_timezone(&Utc);

        // Create repository ID
        let repository_id = RepositoryId::new(node.owner.login, node.name);

        // Convert primary language
        let language = node.primary_language.map(|lang| lang.name);

        // Convert default branch
        let default_branch = node
            .default_branch_ref
            .map(|branch_ref| Branch::new(branch_ref.name));

        // Convert milestones
        let milestones = node
            .milestones
            .nodes
            .into_iter()
            .map(|milestone| {
                let due_date = milestone
                    .due_on
                    .and_then(|date_str| chrono::DateTime::parse_from_rfc3339(&date_str).ok())
                    .map(|date| date.with_timezone(&Utc));

                RepositoryMilestone {
                    milestone_number: MilestoneNumber(milestone.number as u64),
                    milestone_name: MilestoneName(milestone.title),
                    due_date,
                }
            })
            .collect();

        // Convert labels
        let labels = node
            .labels
            .nodes
            .into_iter()
            .map(|label_node| Label::new(label_node.name))
            .collect();

        // Convert mentionable users
        let users = node
            .mentionable_users
            .nodes
            .into_iter()
            .map(|user_node| User::new(user_node.login))
            .collect();

        // Convert releases
        let releases = node
            .releases
            .nodes
            .into_iter()
            .map(|release_node| {
                let created_at = chrono::DateTime::parse_from_rfc3339(&release_node.created_at)
                    .context("Failed to parse release created_at timestamp")
                    .unwrap_or_else(|_| Utc::now().into())
                    .with_timezone(&Utc);

                let published_at = release_node
                    .published_at
                    .and_then(|date_str| chrono::DateTime::parse_from_rfc3339(&date_str).ok())
                    .map(|date| date.with_timezone(&Utc));

                let release_name = release_node
                    .name
                    .unwrap_or_else(|| release_node.tag_name.clone());

                let release_id = ReleaseId(release_name.clone());

                let author = release_node
                    .author
                    .map(|author_node| User::new(author_node.login));

                RepositoryRelease {
                    release_id,
                    name: ReleaseName(release_name),
                    tag_name: TagName(release_node.tag_name),
                    description: release_node.description,
                    created_at,
                    published_at,
                    is_prerelease: release_node.is_prerelease,
                    is_draft: release_node.is_draft,
                    author,
                    url: release_node.url,
                }
            })
            .collect();

        Ok(GithubRepository::new(
            repository_id,
            node.description,
            language,
            created_at,
            updated_at,
            milestones,
            default_branch,
            labels,
            users,
            releases,
        ))
    }
}
