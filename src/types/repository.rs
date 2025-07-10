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

/// Repository milestone association mapping milestone IDs to names.
///
/// This struct represents the relationship between a repository and its milestones,
/// storing both the numeric milestone ID and the human-readable milestone name.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepositoryMilestone {
    /// The numeric milestone identifier as assigned by GitHub
    pub milestone_id: u64,
    /// The human-readable milestone name as displayed in GitHub
    pub milestone_name: String,
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
pub struct GitRepository {
    pub git_repository_id: RepositoryId,
    pub description: Option<String>,
    pub language: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub milestones: Vec<RepositoryMilestone>,
}

impl GitRepository {
    /// Create new repository with basic metadata
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        git_repository_id: RepositoryId,
        description: Option<String>,
        language: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            git_repository_id,
            description,
            language,
            created_at,
            updated_at,
            milestones: Vec::new(),
        }
    }

    /// Create new repository with milestones
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_milestones(
        git_repository_id: RepositoryId,
        description: Option<String>,
        language: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        milestones: Vec<RepositoryMilestone>,
    ) -> Self {
        Self {
            git_repository_id,
            description,
            language,
            created_at,
            updated_at,
            milestones,
        }
    }

    /// Get repository identifier
    pub fn repository_id(&self) -> RepositoryId {
        self.git_repository_id.clone()
    }
}
