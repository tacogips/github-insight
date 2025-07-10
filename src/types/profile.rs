use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::types::{ProjectId, RepositoryId};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct ProfileName(pub String);

impl ProfileName {
    pub const DEFAULT_PROFILE_NAME: &'static str = "default";

    pub fn value(&self) -> &str {
        &self.0
    }
}

impl Default for ProfileName {
    fn default() -> Self {
        Self(Self::DEFAULT_PROFILE_NAME.to_string())
    }
}

impl fmt::Display for ProfileName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for ProfileName {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Profile name wrapper type for database isolation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct ProfileInfo {
    /// Profile name
    pub name: ProfileName,
    /// Profile description
    pub description: Option<String>,
    pub repositories: Vec<RepositoryId>,
    pub projects: Vec<ProjectId>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last modified timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl ProfileInfo {
    /// Create a new GitInsightProfile
    pub fn new(name: ProfileName, description: Option<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            name,
            description,
            repositories: Vec::new(),
            projects: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Update the timestamp
    pub fn touch(&mut self) {
        self.updated_at = chrono::Utc::now();
    }

    /// Add a repository to the profile
    pub fn add_repository(&mut self, repository_id: RepositoryId) {
        if !self.repositories.contains(&repository_id) {
            self.repositories.push(repository_id);
        }
    }

    /// Remove a repository from the profile
    pub fn remove_repository(&mut self, repository_id: &RepositoryId) {
        self.repositories.retain(|r| r != repository_id);
    }

    /// Check if profile contains a repository
    pub fn has_repository(&self, repository_id: &RepositoryId) -> bool {
        self.repositories.contains(repository_id)
    }

    /// Get all repositories in the profile
    pub fn repositories(&self) -> &Vec<RepositoryId> {
        &self.repositories
    }

    /// Add a project to the profile
    pub fn add_project(&mut self, project_id: ProjectId) {
        if !self.projects.contains(&project_id) {
            self.projects.push(project_id);
        }
    }

    /// Remove a project from the profile
    pub fn remove_project(&mut self, project_id: &ProjectId) {
        self.projects.retain(|p| p != project_id);
    }

    /// Check if profile contains a project
    pub fn has_project(&self, project_id: &ProjectId) -> bool {
        self.projects.contains(project_id)
    }

    /// Get all projects in the profile
    pub fn projects(&self) -> &Vec<ProjectId> {
        &self.projects
    }

    /// Get the total number of items in the profile
    pub fn total_items(&self) -> usize {
        self.repositories.len() + self.projects.len()
    }

    /// Check if the profile is empty
    pub fn is_empty(&self) -> bool {
        self.repositories.is_empty() && self.projects.is_empty()
    }
}

impl Default for ProfileInfo {
    fn default() -> Self {
        Self::new(ProfileName("default".to_string()), None)
    }
}
