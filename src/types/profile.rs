use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use crate::types::repository::{Owner, RepositoryName};
use crate::types::{Branch, ProjectId, RepositoryId};

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

/// Repository and branch pair identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryBranchPair {
    pub repository_id: RepositoryId,
    pub branch: Branch,
}

impl RepositoryBranchPair {
    pub fn new(repository_id: RepositoryId, branch: Branch) -> Self {
        Self {
            repository_id,
            branch,
        }
    }

    /// Parse a single repository branch specifier from string in format "repo_url@branch"
    pub fn try_from_str(specifier: &str) -> anyhow::Result<Self> {
        let parts: Vec<&str> = specifier.split('@').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid repository branch specifier format '{}'. Expected format: 'repo_url@branch'",
                specifier
            ));
        }

        let repo_url = parts[0].trim();
        let branch_name = parts[1].trim();

        if repo_url.is_empty() {
            return Err(anyhow::anyhow!(
                "Repository URL cannot be empty in specifier '{}'",
                specifier
            ));
        }

        if branch_name.is_empty() {
            return Err(anyhow::anyhow!(
                "Branch name cannot be empty in specifier '{}'",
                specifier
            ));
        }

        let repository_id = Self::parse_repository_url(repo_url)?;
        let branch = Branch::new(branch_name);

        Ok(Self::new(repository_id, branch))
    }

    /// Parse multiple repository branch specifiers from strings in format "repo_url@branch"
    pub fn try_from_specifiers(specifiers: &[String]) -> anyhow::Result<Vec<Self>> {
        let mut parsed_specifiers = Vec::new();

        for specifier in specifiers {
            let pair = Self::try_from_str(specifier.trim())?;
            parsed_specifiers.push(pair);
        }

        Ok(parsed_specifiers)
    }

    /// Parse repository URL into RepositoryId
    /// This is a helper function for URL parsing
    fn parse_repository_url(url: &str) -> anyhow::Result<RepositoryId> {
        // Simple URL parsing for GitHub URLs
        if let Some(captures) =
            regex::Regex::new(r"https://github\.com/([^/]+)/([^/]+?)(?:\.git)?/?$")
                .unwrap()
                .captures(url)
        {
            let owner = captures.get(1).unwrap().as_str();
            let repo_name = captures.get(2).unwrap().as_str();

            Ok(RepositoryId {
                owner: Owner::from(owner),
                repository_name: RepositoryName::from(repo_name),
            })
        } else {
            Err(anyhow::anyhow!("Invalid repository URL format: {}", url))
        }
    }
}

impl fmt::Display for RepositoryBranchPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.repository_id, self.branch.as_str())
    }
}

/// Group name wrapper type for repository branch groups
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct GroupName(pub String);

impl GroupName {
    pub fn new(name: String) -> Self {
        Self(name)
    }

    pub fn value(&self) -> &str {
        &self.0
    }

    /// Generate default group name with yyyymmdd + hash format
    pub fn generate_default() -> Self {
        let now = Utc::now();
        let date_str = now.format("%Y%m%d").to_string();
        let hash = format!("{:x}", now.timestamp_nanos_opt().unwrap_or(0) % 0xffff);
        Self(format!("{}-{}", date_str, hash))
    }
}

impl fmt::Display for GroupName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for GroupName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for GroupName {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Repository branch group containing multiple repository-branch pairs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepositoryBranchGroup {
    pub name: GroupName,
    pub pairs: Vec<RepositoryBranchPair>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RepositoryBranchGroup {
    pub fn new(name: Option<GroupName>, pairs: Vec<RepositoryBranchPair>) -> Self {
        let now = Utc::now();
        let group_name = name.unwrap_or_else(GroupName::generate_default);

        Self {
            name: group_name,
            pairs,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    pub fn add_pair(&mut self, pair: RepositoryBranchPair) {
        if !self.pairs.contains(&pair) {
            self.pairs.push(pair);
            self.touch();
        }
    }

    pub fn remove_pair(&mut self, pair: &RepositoryBranchPair) {
        let original_len = self.pairs.len();
        self.pairs.retain(|p| p != pair);
        if self.pairs.len() != original_len {
            self.touch();
        }
    }

    pub fn rename(&mut self, new_name: GroupName) {
        self.name = new_name;
        self.touch();
    }

    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    pub fn pair_count(&self) -> usize {
        self.pairs.len()
    }
}

/// Profile name wrapper type for database isolation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProfileInfo {
    /// Profile name
    pub name: ProfileName,
    /// Profile description
    pub description: Option<String>,
    pub repositories: Vec<RepositoryId>,
    pub projects: Vec<ProjectId>,
    /// Repository branch groups organized by group name
    pub repository_branch_groups: HashMap<GroupName, RepositoryBranchGroup>,
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
            repository_branch_groups: HashMap::new(),
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

    /// Add a repository branch group to the profile
    pub fn add_repository_branch_group(&mut self, group: RepositoryBranchGroup) {
        self.repository_branch_groups
            .insert(group.name.clone(), group);
        self.touch();
    }

    /// Remove a repository branch group from the profile
    pub fn remove_repository_branch_group(
        &mut self,
        group_name: &GroupName,
    ) -> Option<RepositoryBranchGroup> {
        let result = self.repository_branch_groups.remove(group_name);
        if result.is_some() {
            self.touch();
        }
        result
    }

    /// Get a repository branch group by name
    pub fn get_repository_branch_group(
        &self,
        group_name: &GroupName,
    ) -> Option<&RepositoryBranchGroup> {
        self.repository_branch_groups.get(group_name)
    }

    /// Get a mutable reference to a repository branch group by name
    pub fn get_repository_branch_group_mut(
        &mut self,
        group_name: &GroupName,
    ) -> Option<&mut RepositoryBranchGroup> {
        self.repository_branch_groups.get_mut(group_name)
    }

    /// Check if profile contains a repository branch group
    pub fn has_repository_branch_group(&self, group_name: &GroupName) -> bool {
        self.repository_branch_groups.contains_key(group_name)
    }

    /// Get all repository branch groups in the profile
    pub fn repository_branch_groups(&self) -> &HashMap<GroupName, RepositoryBranchGroup> {
        &self.repository_branch_groups
    }

    /// List all group names
    pub fn repository_branch_group_names(&self) -> Vec<&GroupName> {
        self.repository_branch_groups.keys().collect()
    }

    /// Remove repository branch groups older than N days
    pub fn remove_groups_older_than(&mut self, days: i64) -> Vec<GroupName> {
        let cutoff_time = chrono::Utc::now() - chrono::Duration::days(days);
        let mut removed_groups = Vec::new();

        self.repository_branch_groups.retain(|name, group| {
            if group.created_at < cutoff_time {
                removed_groups.push(name.clone());
                false
            } else {
                true
            }
        });

        if !removed_groups.is_empty() {
            self.touch();
        }

        removed_groups
    }

    /// Rename a repository branch group
    pub fn rename_repository_branch_group(
        &mut self,
        old_name: &GroupName,
        new_name: GroupName,
    ) -> Result<(), String> {
        if self.repository_branch_groups.contains_key(&new_name) {
            return Err(format!("Group with name '{}' already exists", new_name));
        }

        if let Some(mut group) = self.repository_branch_groups.remove(old_name) {
            group.rename(new_name.clone());
            self.repository_branch_groups.insert(new_name, group);
            self.touch();
            Ok(())
        } else {
            Err(format!("Group with name '{}' not found", old_name))
        }
    }

    /// Get the total number of items in the profile
    pub fn total_items(&self) -> usize {
        self.repositories.len() + self.projects.len() + self.repository_branch_groups.len()
    }

    /// Check if the profile is empty
    pub fn is_empty(&self) -> bool {
        self.repositories.is_empty()
            && self.projects.is_empty()
            && self.repository_branch_groups.is_empty()
    }
}

impl Default for ProfileInfo {
    fn default() -> Self {
        Self::new(ProfileName("default".to_string()), None)
    }
}
