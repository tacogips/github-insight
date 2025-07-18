use crate::github::graphql::graphql_types::LabelsConnection;
use serde::{Deserialize, Serialize};

/// Wrapper type for milestone numbers providing type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MilestoneNumber(pub u64);

impl MilestoneNumber {
    /// Create a new milestone number
    pub fn new(number: u64) -> Self {
        Self(number)
    }

    /// Get the inner value
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for MilestoneNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub owner: RepositoryOwner,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryOwner {
    pub login: String,
}

/// GraphQL response type for a single repository query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryResponse {
    pub repository: Option<RepositoryNode>,
}

/// Repository node from GraphQL response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryNode {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "primaryLanguage")]
    pub primary_language: Option<PrimaryLanguage>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    #[serde(rename = "defaultBranchRef")]
    pub default_branch_ref: Option<BranchRef>,
    pub milestones: MilestonesConnection,
    pub labels: LabelsConnection,
    pub owner: RepositoryOwner,
    #[serde(rename = "mentionableUsers")]
    pub mentionable_users: MentionableUsersConnection,
    pub releases: ReleasesConnection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimaryLanguage {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchRef {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestonesConnection {
    pub nodes: Vec<MilestoneNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneNode {
    pub number: u64,
    pub title: String,
    #[serde(rename = "dueOn")]
    pub due_on: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MentionableUsersConnection {
    pub nodes: Vec<MentionableUserNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MentionableUserNode {
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelNode {
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleasesConnection {
    pub nodes: Vec<ReleaseNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseNode {
    pub name: Option<String>,
    #[serde(rename = "tagName")]
    pub tag_name: String,
    pub description: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "publishedAt")]
    pub published_at: Option<String>,
    #[serde(rename = "isPrerelease")]
    pub is_prerelease: bool,
    #[serde(rename = "isDraft")]
    pub is_draft: bool,
    pub author: Option<ReleaseAuthor>,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseAuthor {
    pub login: String,
    pub name: Option<String>,
}
