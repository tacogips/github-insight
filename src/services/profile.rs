//! Profile management service
//!
//! This service handles profile registration, management, and persistence operations.
//! It manages repositories and projects within profiles, providing the core business
//! logic for profile-based organization of GitHub resources.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::types::{
    GroupName, ProfileInfo, ProfileName, ProjectId, RepositoryBranchGroup, RepositoryBranchUnit,
    RepositoryId,
};

/// Profile management service for handling repository and project organization
#[derive(Debug, Clone)]
pub struct ProfileService {
    /// In-memory profile storage
    profiles: HashMap<ProfileName, ProfileInfo>,
    /// data directory path
    data_dir: PathBuf,
}

/// Profile service errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileServiceError {
    /// Profile already exists
    ProfileAlreadyExists(String),
    /// Profile not found
    ProfileNotFound(String),
    /// Repository already exists in profile
    RepositoryAlreadyExists(String),
    /// Repository not found in profile
    RepositoryNotFound(String),
    /// Project already exists in profile
    ProjectAlreadyExists(String),
    /// Project not found in profile
    ProjectNotFound(String),
    /// Repository branch group already exists
    GroupAlreadyExists(String),
    /// Repository branch group not found
    GroupNotFound(String),
    /// Repository branch unit already exists in group
    UnitAlreadyExists(String),
    /// Repository branch unit not found in group
    UnitNotFound(String),
    /// Invalid group name
    InvalidGroupName(String),
    /// Invalid profile name
    InvalidProfileName(String),
    /// IO error during persistence
    IoError(String),
    /// Serialization error
    SerializationError(String),
}

impl std::fmt::Display for ProfileServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProfileAlreadyExists(name) => write!(f, "Profile '{}' already exists", name),
            Self::ProfileNotFound(name) => write!(f, "Profile '{}' not found", name),
            Self::RepositoryAlreadyExists(repo) => {
                write!(f, "Repository '{}' already exists in profile", repo)
            }
            Self::RepositoryNotFound(repo) => {
                write!(f, "Repository '{}' not found in profile", repo)
            }
            Self::ProjectAlreadyExists(project) => {
                write!(f, "Project '{}' already exists in profile", project)
            }
            Self::ProjectNotFound(project) => {
                write!(f, "Project '{}' not found in profile", project)
            }
            Self::GroupAlreadyExists(group) => {
                write!(f, "Repository branch group '{}' already exists", group)
            }
            Self::GroupNotFound(group) => {
                write!(f, "Repository branch group '{}' not found", group)
            }
            Self::UnitAlreadyExists(unit) => {
                write!(
                    f,
                    "Repository branch unit '{}' already exists in group",
                    unit
                )
            }
            Self::UnitNotFound(unit) => {
                write!(f, "Repository branch unit '{}' not found in group", unit)
            }
            Self::InvalidGroupName(name) => write!(f, "Invalid group name: '{}'", name),
            Self::InvalidProfileName(name) => write!(f, "Invalid profile name: '{}'", name),
            Self::IoError(msg) => write!(f, "IO error: {}", msg),
            Self::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for ProfileServiceError {}

impl ProfileService {
    /// Create a new profile service with the specified configuration directory
    pub fn new(data_dir: PathBuf) -> Result<Self, ProfileServiceError> {
        let mut service = Self {
            profiles: HashMap::new(),
            data_dir,
        };

        // Create configuration directory if it doesn't exist
        service.ensure_config_directory()?;

        // Load existing profiles from disk
        service.load_all_profiles()?;

        // Ensure default profile exists
        if !service.profiles.contains_key(&ProfileName::default()) {
            service.create_profile(&ProfileName::default(), None)?;
        }

        Ok(service)
    }

    /// Create a new profile
    pub fn create_profile(
        &mut self,
        name: &ProfileName,
        description: Option<String>,
    ) -> Result<(), ProfileServiceError> {
        // Validate profile name
        self.validate_profile_name(name)?;

        // Check if profile already exists
        if self.profiles.contains_key(name) {
            return Err(ProfileServiceError::ProfileAlreadyExists(name.to_string()));
        }

        // Create new profile
        let profile = ProfileInfo::new(name.clone(), description);
        self.profiles.insert(name.clone(), profile.clone());

        // Persist to disk
        self.save_profile(name, &profile)?;

        Ok(())
    }

    /// Register a repository to a profile
    pub fn register_repository(
        &mut self,
        profile_name: &ProfileName,
        repository_id: RepositoryId,
    ) -> Result<(), ProfileServiceError> {
        // Get or create profile
        let profile = self.get_or_create_profile(profile_name)?;

        // Check if repository already exists
        if profile.has_repository(&repository_id) {
            return Err(ProfileServiceError::RepositoryAlreadyExists(
                repository_id.to_string(),
            ));
        }

        // Add repository to profile
        profile.add_repository(repository_id.clone());

        // Update profile info and persist
        self.update_profile_timestamp(profile_name)?;

        Ok(())
    }

    /// Unregister a repository from a profile
    pub fn unregister_repository(
        &mut self,
        profile_name: &ProfileName,
        repository_id: &RepositoryId,
    ) -> Result<(), ProfileServiceError> {
        {
            let profile = self
                .profiles
                .get_mut(profile_name)
                .ok_or_else(|| ProfileServiceError::ProfileNotFound(profile_name.to_string()))?;

            // Check if repository exists
            if !profile.has_repository(repository_id) {
                return Err(ProfileServiceError::RepositoryNotFound(
                    repository_id.to_string(),
                ));
            }

            // Remove repository from profile
            profile.remove_repository(repository_id);
        }

        // Update profile info and persist
        self.update_profile_timestamp(profile_name)?;

        Ok(())
    }

    /// Register a project to a profile
    pub fn register_project(
        &mut self,
        profile_name: &ProfileName,
        project_id: ProjectId,
    ) -> Result<(), ProfileServiceError> {
        // Get or create profile
        let profile = self.get_or_create_profile(profile_name)?;

        // Check if project already exists
        if profile.has_project(&project_id) {
            return Err(ProfileServiceError::ProjectAlreadyExists(
                project_id.to_string(),
            ));
        }

        // Add project to profile
        profile.add_project(project_id.clone());

        // Update profile info and persist
        self.update_profile_timestamp(profile_name)?;

        Ok(())
    }

    /// Unregister a project from a profile
    pub fn unregister_project(
        &mut self,
        profile_name: &ProfileName,
        project_id: &ProjectId,
    ) -> Result<(), ProfileServiceError> {
        {
            let profile = self
                .profiles
                .get_mut(profile_name)
                .ok_or_else(|| ProfileServiceError::ProfileNotFound(profile_name.to_string()))?;

            // Check if project exists
            if !profile.has_project(project_id) {
                return Err(ProfileServiceError::ProjectNotFound(project_id.to_string()));
            }

            // Remove project from profile
            profile.remove_project(project_id);
        }

        // Update profile info and persist
        self.update_profile_timestamp(profile_name)?;

        Ok(())
    }

    /// List all repositories in a profile
    pub fn list_repositories(
        &self,
        profile_name: &ProfileName,
    ) -> Result<Vec<RepositoryId>, ProfileServiceError> {
        let profile = self
            .profiles
            .get(profile_name)
            .ok_or_else(|| ProfileServiceError::ProfileNotFound(profile_name.to_string()))?;

        Ok(profile.repositories().clone())
    }

    /// List all projects in a profile
    pub fn list_projects(
        &self,
        profile_name: &ProfileName,
    ) -> Result<Vec<ProjectId>, ProfileServiceError> {
        let profile = self
            .profiles
            .get(profile_name)
            .ok_or_else(|| ProfileServiceError::ProfileNotFound(profile_name.to_string()))?;

        Ok(profile.projects().clone())
    }

    /// Register a repository branch group to a profile
    pub fn register_repository_branch_group(
        &mut self,
        profile_name: &ProfileName,
        group_name: Option<GroupName>,
        units: Vec<RepositoryBranchUnit>,
    ) -> Result<GroupName, ProfileServiceError> {
        // Get or create profile
        let profile = self.get_or_create_profile(profile_name)?;

        // Create the group
        let group = RepositoryBranchGroup::new(group_name, units);
        let final_group_name = group.name.clone();

        // Check if group already exists
        if profile.has_repository_branch_group(&final_group_name) {
            return Err(ProfileServiceError::GroupAlreadyExists(
                final_group_name.to_string(),
            ));
        }

        // Add group to profile
        profile.add_repository_branch_group(group);

        // Update profile info and persist
        self.update_profile_timestamp(profile_name)?;

        Ok(final_group_name)
    }

    /// Unregister a repository branch group from a profile
    pub fn unregister_repository_branch_group(
        &mut self,
        profile_name: &ProfileName,
        group_name: &GroupName,
    ) -> Result<RepositoryBranchGroup, ProfileServiceError> {
        let group = {
            let profile = self
                .profiles
                .get_mut(profile_name)
                .ok_or_else(|| ProfileServiceError::ProfileNotFound(profile_name.to_string()))?;

            profile
                .remove_repository_branch_group(group_name)
                .ok_or_else(|| ProfileServiceError::GroupNotFound(group_name.to_string()))?
        };

        // Update profile info and persist
        self.update_profile_timestamp(profile_name)?;

        Ok(group)
    }

    /// Add a repository branch unit to an existing group
    pub fn add_unit_to_group(
        &mut self,
        profile_name: &ProfileName,
        group_name: &GroupName,
        unit: RepositoryBranchUnit,
    ) -> Result<(), ProfileServiceError> {
        {
            let profile = self
                .profiles
                .get_mut(profile_name)
                .ok_or_else(|| ProfileServiceError::ProfileNotFound(profile_name.to_string()))?;

            let group = profile
                .get_repository_branch_group_mut(group_name)
                .ok_or_else(|| ProfileServiceError::GroupNotFound(group_name.to_string()))?;

            // Check if unit already exists
            if group.units.contains(&unit) {
                return Err(ProfileServiceError::UnitAlreadyExists(unit.to_string()));
            }

            group.add_unit(unit);
        }

        // Update profile info and persist
        self.update_profile_timestamp(profile_name)?;

        Ok(())
    }

    /// Remove a repository branch unit from a group
    pub fn remove_unit_from_group(
        &mut self,
        profile_name: &ProfileName,
        group_name: &GroupName,
        unit: &RepositoryBranchUnit,
    ) -> Result<(), ProfileServiceError> {
        {
            let profile = self
                .profiles
                .get_mut(profile_name)
                .ok_or_else(|| ProfileServiceError::ProfileNotFound(profile_name.to_string()))?;

            let group = profile
                .get_repository_branch_group_mut(group_name)
                .ok_or_else(|| ProfileServiceError::GroupNotFound(group_name.to_string()))?;

            // Check if unit exists
            if !group.units.contains(unit) {
                return Err(ProfileServiceError::UnitNotFound(unit.to_string()));
            }

            group.remove_unit(unit);
        }

        // Update profile info and persist
        self.update_profile_timestamp(profile_name)?;

        Ok(())
    }

    /// Rename a repository branch group
    pub fn rename_repository_branch_group(
        &mut self,
        profile_name: &ProfileName,
        old_name: &GroupName,
        new_name: GroupName,
    ) -> Result<(), ProfileServiceError> {
        {
            let profile = self
                .profiles
                .get_mut(profile_name)
                .ok_or_else(|| ProfileServiceError::ProfileNotFound(profile_name.to_string()))?;

            profile
                .rename_repository_branch_group(old_name, new_name)
                .map_err(|e| ProfileServiceError::InvalidGroupName(e))?;
        }

        // Update profile info and persist
        self.update_profile_timestamp(profile_name)?;

        Ok(())
    }

    /// List all repository branch groups in a profile
    pub fn list_repository_branch_groups(
        &self,
        profile_name: &ProfileName,
    ) -> Result<Vec<GroupName>, ProfileServiceError> {
        let profile = self
            .profiles
            .get(profile_name)
            .ok_or_else(|| ProfileServiceError::ProfileNotFound(profile_name.to_string()))?;

        Ok(profile
            .repository_branch_group_names()
            .into_iter()
            .cloned()
            .collect())
    }

    /// Get a specific repository branch group
    pub fn get_repository_branch_group(
        &self,
        profile_name: &ProfileName,
        group_name: &GroupName,
    ) -> Result<RepositoryBranchGroup, ProfileServiceError> {
        let profile = self
            .profiles
            .get(profile_name)
            .ok_or_else(|| ProfileServiceError::ProfileNotFound(profile_name.to_string()))?;

        profile
            .get_repository_branch_group(group_name)
            .cloned()
            .ok_or_else(|| ProfileServiceError::GroupNotFound(group_name.to_string()))
    }

    /// Remove repository branch groups older than N days
    pub fn remove_groups_older_than(
        &mut self,
        profile_name: &ProfileName,
        days: i64,
    ) -> Result<Vec<GroupName>, ProfileServiceError> {
        let removed_groups = {
            let profile = self
                .profiles
                .get_mut(profile_name)
                .ok_or_else(|| ProfileServiceError::ProfileNotFound(profile_name.to_string()))?;

            profile.remove_groups_older_than(days)
        };

        if !removed_groups.is_empty() {
            // Update profile info and persist
            self.update_profile_timestamp(profile_name)?;
        }

        Ok(removed_groups)
    }

    /// List all profile names
    pub fn list_profiles(&self) -> Vec<ProfileName> {
        self.profiles.keys().cloned().collect()
    }

    /// Get profile information including metadata
    pub fn get_profile_info(
        &self,
        profile_name: &ProfileName,
    ) -> Result<ProfileInfo, ProfileServiceError> {
        self.load_profile(profile_name)
    }

    /// Delete a profile
    pub fn delete_profile(
        &mut self,
        profile_name: &ProfileName,
    ) -> Result<(), ProfileServiceError> {
        // Don't allow deleting the default profile
        if profile_name == &ProfileName::default() {
            return Err(ProfileServiceError::InvalidProfileName(
                "Cannot delete default profile".to_string(),
            ));
        }

        // Remove from memory
        if self.profiles.remove(profile_name).is_none() {
            return Err(ProfileServiceError::ProfileNotFound(
                profile_name.to_string(),
            ));
        }

        // Remove from disk
        let profile_file = self.get_profile_file_path(profile_name);
        if profile_file.exists() {
            std::fs::remove_file(profile_file)
                .map_err(|e| ProfileServiceError::IoError(e.to_string()))?;
        }

        Ok(())
    }

    /// Get or create a profile (used internally)
    fn get_or_create_profile(
        &mut self,
        profile_name: &ProfileName,
    ) -> Result<&mut ProfileInfo, ProfileServiceError> {
        if !self.profiles.contains_key(profile_name) {
            self.create_profile(profile_name, None)?;
        }
        Ok(self.profiles.get_mut(profile_name).unwrap())
    }

    /// Validate profile name
    fn validate_profile_name(&self, name: &ProfileName) -> Result<(), ProfileServiceError> {
        if name.value().is_empty() || name.value().len() > 100 {
            return Err(ProfileServiceError::InvalidProfileName(
                "Profile name must be 1-100 characters".to_string(),
            ));
        }

        // Check for invalid characters
        if name
            .value()
            .contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|'])
        {
            return Err(ProfileServiceError::InvalidProfileName(
                "Profile name contains invalid characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Ensure configuration directory exists
    fn ensure_config_directory(&self) -> Result<(), ProfileServiceError> {
        std::fs::create_dir_all(&self.data_dir)
            .map_err(|e| ProfileServiceError::IoError(e.to_string()))?;
        Ok(())
    }

    /// Get the file path for a profile
    fn get_profile_file_path(&self, profile_name: &ProfileName) -> PathBuf {
        self.data_dir.join(format!("{}.toml", profile_name))
    }

    /// Save profile to disk
    fn save_profile(
        &self,
        profile_name: &ProfileName,
        profile: &ProfileInfo,
    ) -> Result<(), ProfileServiceError> {
        let profile_file = self.get_profile_file_path(profile_name);
        let toml_content = toml::to_string(profile)
            .map_err(|e| ProfileServiceError::SerializationError(e.to_string()))?;

        std::fs::write(profile_file, toml_content)
            .map_err(|e| ProfileServiceError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Load profile from disk
    fn load_profile(&self, profile_name: &ProfileName) -> Result<ProfileInfo, ProfileServiceError> {
        let profile_file = self.get_profile_file_path(profile_name);

        if !profile_file.exists() {
            // Return current profile if file doesn't exist
            let profile = self
                .profiles
                .get(profile_name)
                .ok_or_else(|| ProfileServiceError::ProfileNotFound(profile_name.to_string()))?;
            return Ok(profile.clone());
        }

        let content = std::fs::read_to_string(profile_file)
            .map_err(|e| ProfileServiceError::IoError(e.to_string()))?;

        let profile: ProfileInfo = toml::from_str(&content)
            .map_err(|e| ProfileServiceError::SerializationError(e.to_string()))?;

        Ok(profile)
    }

    /// Load all profiles from disk
    fn load_all_profiles(&mut self) -> Result<(), ProfileServiceError> {
        if !self.data_dir.exists() {
            return Ok(());
        }

        let entries = std::fs::read_dir(&self.data_dir)
            .map_err(|e| ProfileServiceError::IoError(e.to_string()))?;

        for entry in entries {
            let entry = entry.map_err(|e| ProfileServiceError::IoError(e.to_string()))?;
            let path = entry.path();

            if path.extension().and_then(|ext| ext.to_str()) == Some("toml") {
                if let Some(profile_name) = path.file_stem().and_then(|s| s.to_str()) {
                    let profile_name = ProfileName::from(profile_name);
                    if let Ok(profile) = self.load_profile(&profile_name) {
                        self.profiles.insert(profile_name, profile);
                    }
                }
            }
        }

        Ok(())
    }

    /// Update profile timestamp and persist
    fn update_profile_timestamp(
        &mut self,
        profile_name: &ProfileName,
    ) -> Result<(), ProfileServiceError> {
        {
            let profile = self
                .profiles
                .get_mut(profile_name)
                .ok_or_else(|| ProfileServiceError::ProfileNotFound(profile_name.to_string()))?;
            profile.touch();
        }
        let profile = self.profiles.get(profile_name).unwrap().clone();
        self.save_profile(profile_name, &profile)?;
        Ok(())
    }
}

/// Get the default configuration directory for profiles
///
/// Returns `~/.local/share/github-insight/profiles/` on Unix-like systems
pub fn default_profile_config_dir() -> Result<PathBuf, ProfileServiceError> {
    let home_dir = dirs::home_dir().ok_or_else(|| {
        ProfileServiceError::IoError("Unable to determine home directory".to_string())
    })?;

    #[cfg(unix)]
    let config_dir = home_dir.join(".local/share/github-insight/profiles");

    #[cfg(windows)]
    let config_dir = home_dir.join("AppData/Roaming/github-insight/profiles");

    #[cfg(target_os = "macos")]
    let config_dir = home_dir.join("Library/Application Support/github-insight/profiles");

    Ok(config_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        RepositoryId,
        repository::{Owner, RepositoryName},
    };
    use tempfile::TempDir;

    #[test]
    fn test_create_profile_service() {
        let temp_dir = TempDir::new().unwrap();
        let service = ProfileService::new(temp_dir.path().to_path_buf());
        assert!(service.is_ok());
    }

    #[test]
    fn test_create_and_list_profiles() {
        let temp_dir = TempDir::new().unwrap();
        let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

        service
            .create_profile(&ProfileName::from("test"), Some("Test profile".to_string()))
            .unwrap();
        let profiles = service.list_profiles();
        assert!(profiles.contains(&ProfileName::from("test")));
        assert!(profiles.contains(&ProfileName::from("default")));
    }

    #[test]
    fn test_repository_registration() {
        let temp_dir = TempDir::new().unwrap();
        let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

        let repo_id = RepositoryId {
            owner: Owner::from("test-owner"),
            repository_name: RepositoryName::from("test-repo"),
        };

        service
            .register_repository(&ProfileName::from("default"), repo_id.clone())
            .unwrap();
        let repos = service
            .list_repositories(&ProfileName::from("default"))
            .unwrap();
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0], repo_id);
    }

    #[test]
    fn test_repository_branch_group_registration() {
        let temp_dir = TempDir::new().unwrap();
        let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

        let repo_id = RepositoryId {
            owner: Owner::from("test-owner"),
            repository_name: RepositoryName::from("test-repo"),
        };
        let branch = crate::types::Branch::new("main");
        let unit = RepositoryBranchUnit::new(repo_id, branch);

        let group_name = service
            .register_repository_branch_group(
                &ProfileName::from("default"),
                Some(GroupName::from("test-group")),
                vec![unit.clone()],
            )
            .unwrap();

        assert_eq!(group_name.value(), "test-group");

        let groups = service
            .list_repository_branch_groups(&ProfileName::from("default"))
            .unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0], group_name);

        let group = service
            .get_repository_branch_group(&ProfileName::from("default"), &group_name)
            .unwrap();
        assert_eq!(group.units.len(), 1);
        assert_eq!(group.units[0], unit);
    }

    #[test]
    fn test_repository_branch_group_auto_naming() {
        let temp_dir = TempDir::new().unwrap();
        let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

        let repo_id = RepositoryId {
            owner: Owner::from("test-owner"),
            repository_name: RepositoryName::from("test-repo"),
        };
        let branch = crate::types::Branch::new("main");
        let unit = RepositoryBranchUnit::new(repo_id, branch);

        // Register group without providing name
        let group_name = service
            .register_repository_branch_group(&ProfileName::from("default"), None, vec![unit])
            .unwrap();

        // Should generate a name with yyyymmdd-hash format
        assert!(group_name.value().len() > 8); // At minimum yyyymmdd format
        assert!(group_name.value().contains('-')); // Should contain dash separator
    }

    #[test]
    fn test_repository_branch_group_unit_management() {
        let temp_dir = TempDir::new().unwrap();
        let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

        let repo_id1 = RepositoryId {
            owner: Owner::from("test-owner1"),
            repository_name: RepositoryName::from("test-repo1"),
        };
        let repo_id2 = RepositoryId {
            owner: Owner::from("test-owner2"),
            repository_name: RepositoryName::from("test-repo2"),
        };

        let unit1 = RepositoryBranchUnit::new(repo_id1, crate::types::Branch::new("main"));
        let unit2 =
            RepositoryBranchUnit::new(repo_id2.clone(), crate::types::Branch::new("develop"));
        let unit3 = RepositoryBranchUnit::new(repo_id2, crate::types::Branch::new("feature"));

        // Create group with initial unit
        let group_name = service
            .register_repository_branch_group(
                &ProfileName::from("default"),
                Some(GroupName::from("test-group")),
                vec![unit1.clone()],
            )
            .unwrap();

        // Add more units
        service
            .add_unit_to_group(&ProfileName::from("default"), &group_name, unit2.clone())
            .unwrap();
        service
            .add_unit_to_group(&ProfileName::from("default"), &group_name, unit3.clone())
            .unwrap();

        let group = service
            .get_repository_branch_group(&ProfileName::from("default"), &group_name)
            .unwrap();
        assert_eq!(group.units.len(), 3);

        // Remove a unit
        service
            .remove_unit_from_group(&ProfileName::from("default"), &group_name, &unit2)
            .unwrap();

        let group = service
            .get_repository_branch_group(&ProfileName::from("default"), &group_name)
            .unwrap();
        assert_eq!(group.units.len(), 2);
        assert!(group.units.contains(&unit1));
        assert!(group.units.contains(&unit3));
        assert!(!group.units.contains(&unit2));
    }

    #[test]
    fn test_repository_branch_group_rename() {
        let temp_dir = TempDir::new().unwrap();
        let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

        let unit = RepositoryBranchUnit::new(
            RepositoryId {
                owner: Owner::from("test-owner"),
                repository_name: RepositoryName::from("test-repo"),
            },
            crate::types::Branch::new("main"),
        );

        let original_name = service
            .register_repository_branch_group(
                &ProfileName::from("default"),
                Some(GroupName::from("original-name")),
                vec![unit],
            )
            .unwrap();

        // Rename the group
        let new_name = GroupName::from("new-name");
        service
            .rename_repository_branch_group(
                &ProfileName::from("default"),
                &original_name,
                new_name.clone(),
            )
            .unwrap();

        // Original name should not exist
        assert!(
            service
                .get_repository_branch_group(&ProfileName::from("default"), &original_name)
                .is_err()
        );

        // New name should exist
        let group = service
            .get_repository_branch_group(&ProfileName::from("default"), &new_name)
            .unwrap();
        assert_eq!(group.name, new_name);
    }

    #[test]
    fn test_repository_branch_group_cleanup_by_date() {
        let temp_dir = TempDir::new().unwrap();
        let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

        let unit = RepositoryBranchUnit::new(
            RepositoryId {
                owner: Owner::from("test-owner"),
                repository_name: RepositoryName::from("test-repo"),
            },
            crate::types::Branch::new("main"),
        );

        // Create a group
        let group_name = service
            .register_repository_branch_group(
                &ProfileName::from("default"),
                Some(GroupName::from("test-group")),
                vec![unit],
            )
            .unwrap();

        // Cleanup groups older than 1 day (should not remove the newly created group)
        let removed_groups = service
            .remove_groups_older_than(&ProfileName::from("default"), 1)
            .unwrap();
        assert_eq!(removed_groups.len(), 0);

        // Cleanup groups older than 0 days (should remove all groups)
        let removed_groups = service
            .remove_groups_older_than(&ProfileName::from("default"), 0)
            .unwrap();
        assert_eq!(removed_groups.len(), 1);
        assert_eq!(removed_groups[0], group_name);

        // Group should no longer exist
        assert!(
            service
                .get_repository_branch_group(&ProfileName::from("default"), &group_name)
                .is_err()
        );
    }
}
