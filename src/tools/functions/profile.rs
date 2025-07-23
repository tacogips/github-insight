//! Profile management tool functions
//!
//! This module provides MCP tool functions for profile management operations,
//! including creating, listing, and deleting profiles, as well as managing
//! repositories and projects within profiles.

use crate::services::{ProfileService, default_profile_config_dir};
use crate::types::profile::ProfileInfo;
use crate::types::{GroupName, ProfileName, ProjectId, ProjectUrl, RepositoryBranchGroup, RepositoryBranchUnit, RepositoryId, RepositoryUrl};

/// Create a new profile
pub async fn create_profile(
    profile_name: String,
    description: Option<String>,
) -> Result<(), String> {
    let config_dir = default_profile_config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;

    let mut service = ProfileService::new(config_dir)
        .map_err(|e| format!("Failed to create profile service: {}", e))?;

    let profile_name = ProfileName::from(profile_name.as_str());

    service
        .create_profile(&profile_name, description)
        .map_err(|e| format!("Failed to create profile: {}", e))?;

    Ok(())
}

/// List all profiles
pub async fn list_profiles() -> Result<Vec<ProfileName>, String> {
    let config_dir = default_profile_config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;

    let service = ProfileService::new(config_dir)
        .map_err(|e| format!("Failed to create profile service: {}", e))?;

    let profiles = service.list_profiles();

    Ok(profiles)
}

/// Delete a profile
pub async fn delete_profile(profile_name: String) -> Result<(), String> {
    let config_dir = default_profile_config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;

    let mut service = ProfileService::new(config_dir)
        .map_err(|e| format!("Failed to create profile service: {}", e))?;

    let profile_name = ProfileName::from(profile_name.as_str());

    service
        .delete_profile(&profile_name)
        .map_err(|e| format!("Failed to delete profile: {}", e))?;

    Ok(())
}

/// Register a repository to a profile
pub async fn register_repository(
    profile_name: String,
    repository_id: RepositoryId,
) -> Result<(), String> {
    let config_dir = default_profile_config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;

    let mut service = ProfileService::new(config_dir)
        .map_err(|e| format!("Failed to create profile service: {}", e))?;

    let profile_name = ProfileName::from(profile_name.as_str());

    service
        .register_repository(&profile_name, repository_id.clone())
        .map_err(|e| format!("Failed to register repository: {}", e))?;

    Ok(())
}

/// Unregister a repository from a profile
pub async fn unregister_repository(
    profile_name: String,
    repository_id: RepositoryId,
) -> Result<(), String> {
    let config_dir = default_profile_config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;

    let mut service = ProfileService::new(config_dir)
        .map_err(|e| format!("Failed to create profile service: {}", e))?;

    let profile_name = ProfileName::from(profile_name.as_str());

    service
        .unregister_repository(&profile_name, &repository_id)
        .map_err(|e| format!("Failed to unregister repository: {}", e))?;

    Ok(())
}

/// List repositories in a profile
pub async fn list_repositories(profile_name: String) -> Result<Vec<RepositoryUrl>, String> {
    let config_dir = default_profile_config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;

    let service = ProfileService::new(config_dir)
        .map_err(|e| format!("Failed to create profile service: {}", e))?;

    let profile_name = ProfileName::from(profile_name.as_str());

    let repositories = service
        .list_repositories(&profile_name)
        .map_err(|e| format!("Failed to list repositories: {}", e))?;

    // Convert RepositoryId to RepositoryUrl
    let repository_urls = repositories
        .into_iter()
        .map(|repo_id| RepositoryUrl(repo_id.url()))
        .collect();

    Ok(repository_urls)
}

/// Register a project to a profile
pub async fn register_project(profile_name: String, project_id: ProjectId) -> Result<(), String> {
    let config_dir = default_profile_config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;

    let mut service = ProfileService::new(config_dir)
        .map_err(|e| format!("Failed to create profile service: {}", e))?;

    let profile_name = ProfileName::from(profile_name.as_str());

    service
        .register_project(&profile_name, project_id.clone())
        .map_err(|e| format!("Failed to register project: {}", e))?;

    Ok(())
}

/// Unregister a project from a profile
pub async fn unregister_project(profile_name: String, project_id: ProjectId) -> Result<(), String> {
    let config_dir = default_profile_config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;

    let mut service = ProfileService::new(config_dir)
        .map_err(|e| format!("Failed to create profile service: {}", e))?;

    let profile_name = ProfileName::from(profile_name.as_str());

    service
        .unregister_project(&profile_name, &project_id)
        .map_err(|e| format!("Failed to unregister project: {}", e))?;

    Ok(())
}

/// List projects in a profile
pub async fn list_projects(profile_name: String) -> Result<Vec<ProjectUrl>, String> {
    let config_dir = default_profile_config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;

    let service = ProfileService::new(config_dir)
        .map_err(|e| format!("Failed to create profile service: {}", e))?;

    let profile_name = ProfileName::from(profile_name.as_str());

    let projects = service
        .list_projects(&profile_name)
        .map_err(|e| format!("Failed to list projects: {}", e))?;

    // Convert ProjectId to ProjectUrl
    let project_urls = projects
        .into_iter()
        .map(|project_id| ProjectUrl(project_id.url()))
        .collect();

    Ok(project_urls)
}

/// Get profile information
pub async fn get_profile_info(profile_name: String) -> Result<ProfileInfo, String> {
    let config_dir = default_profile_config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;

    let service = ProfileService::new(config_dir)
        .map_err(|e| format!("Failed to create profile service: {}", e))?;

    let profile_name = ProfileName::from(profile_name.as_str());

    let profile_info = service
        .get_profile_info(&profile_name)
        .map_err(|e| format!("Failed to get profile info: {}", e))?;

    Ok(profile_info)
}

/// Register a repository branch group to a profile
pub async fn register_repository_branch_group(
    profile_name: String,
    group_name: Option<String>,
    units: Vec<String>,
) -> Result<String, String> {
    let config_dir = default_profile_config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;

    let mut service = ProfileService::new(config_dir)
        .map_err(|e| format!("Failed to create profile service: {}", e))?;

    let profile_name = ProfileName::from(profile_name.as_str());
    let group_name_opt = group_name.map(GroupName::from);
    
    // Parse repository branch units
    let parsed_units = RepositoryBranchUnit::try_from_specifiers(&units)
        .map_err(|e| format!("Failed to parse repository branch units: {}", e))?;

    let final_group_name = service
        .register_repository_branch_group(&profile_name, group_name_opt, parsed_units)
        .map_err(|e| format!("Failed to register repository branch group: {}", e))?;

    Ok(final_group_name.value().to_string())
}

/// Unregister a repository branch group from a profile
pub async fn unregister_repository_branch_group(
    profile_name: String,
    group_name: String,
) -> Result<RepositoryBranchGroup, String> {
    let config_dir = default_profile_config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;

    let mut service = ProfileService::new(config_dir)
        .map_err(|e| format!("Failed to create profile service: {}", e))?;

    let profile_name = ProfileName::from(profile_name.as_str());
    let group_name = GroupName::from(group_name.as_str());

    let removed_group = service
        .unregister_repository_branch_group(&profile_name, &group_name)
        .map_err(|e| format!("Failed to unregister repository branch group: {}", e))?;

    Ok(removed_group)
}

/// Add repository branch units to an existing group
pub async fn add_units_to_group(
    profile_name: String,
    group_name: String,
    units: Vec<String>,
) -> Result<(), String> {
    let config_dir = default_profile_config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;

    let mut service = ProfileService::new(config_dir)
        .map_err(|e| format!("Failed to create profile service: {}", e))?;

    let profile_name = ProfileName::from(profile_name.as_str());
    let group_name = GroupName::from(group_name.as_str());
    
    // Parse repository branch units
    let parsed_units = RepositoryBranchUnit::try_from_specifiers(&units)
        .map_err(|e| format!("Failed to parse repository branch units: {}", e))?;

    for unit in parsed_units {
        service
            .add_unit_to_group(&profile_name, &group_name, unit)
            .map_err(|e| format!("Failed to add unit to group: {}", e))?;
    }

    Ok(())
}

/// Remove repository branch units from a group
pub async fn remove_units_from_group(
    profile_name: String,
    group_name: String,
    units: Vec<String>,
) -> Result<(), String> {
    let config_dir = default_profile_config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;

    let mut service = ProfileService::new(config_dir)
        .map_err(|e| format!("Failed to create profile service: {}", e))?;

    let profile_name = ProfileName::from(profile_name.as_str());
    let group_name = GroupName::from(group_name.as_str());
    
    // Parse repository branch units
    let parsed_units = RepositoryBranchUnit::try_from_specifiers(&units)
        .map_err(|e| format!("Failed to parse repository branch units: {}", e))?;

    for unit in &parsed_units {
        service
            .remove_unit_from_group(&profile_name, &group_name, unit)
            .map_err(|e| format!("Failed to remove unit from group: {}", e))?;
    }

    Ok(())
}

/// Rename a repository branch group
pub async fn rename_repository_branch_group(
    profile_name: String,
    old_name: String,
    new_name: String,
) -> Result<(), String> {
    let config_dir = default_profile_config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;

    let mut service = ProfileService::new(config_dir)
        .map_err(|e| format!("Failed to create profile service: {}", e))?;

    let profile_name = ProfileName::from(profile_name.as_str());
    let old_name = GroupName::from(old_name.as_str());
    let new_name = GroupName::from(new_name.as_str());

    service
        .rename_repository_branch_group(&profile_name, &old_name, new_name)
        .map_err(|e| format!("Failed to rename repository branch group: {}", e))?;

    Ok(())
}

/// List all repository branch groups in a profile
pub async fn list_repository_branch_groups(profile_name: String) -> Result<Vec<String>, String> {
    let config_dir = default_profile_config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;

    let service = ProfileService::new(config_dir)
        .map_err(|e| format!("Failed to create profile service: {}", e))?;

    let profile_name = ProfileName::from(profile_name.as_str());

    let groups = service
        .list_repository_branch_groups(&profile_name)
        .map_err(|e| format!("Failed to list repository branch groups: {}", e))?;

    let group_names = groups
        .into_iter()
        .map(|group_name| group_name.value().to_string())
        .collect();

    Ok(group_names)
}

/// Get a specific repository branch group
pub async fn get_repository_branch_group(
    profile_name: String,
    group_name: String,
) -> Result<RepositoryBranchGroup, String> {
    let config_dir = default_profile_config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;

    let service = ProfileService::new(config_dir)
        .map_err(|e| format!("Failed to create profile service: {}", e))?;

    let profile_name = ProfileName::from(profile_name.as_str());
    let group_name = GroupName::from(group_name.as_str());

    let group = service
        .get_repository_branch_group(&profile_name, &group_name)
        .map_err(|e| format!("Failed to get repository branch group: {}", e))?;

    Ok(group)
}

/// Remove repository branch groups older than N days
pub async fn cleanup_repository_branch_groups(
    profile_name: String,
    days: i64,
) -> Result<Vec<String>, String> {
    let config_dir = default_profile_config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;

    let mut service = ProfileService::new(config_dir)
        .map_err(|e| format!("Failed to create profile service: {}", e))?;

    let profile_name = ProfileName::from(profile_name.as_str());

    let removed_groups = service
        .remove_groups_older_than(&profile_name, days)
        .map_err(|e| format!("Failed to cleanup repository branch groups: {}", e))?;

    let removed_group_names = removed_groups
        .into_iter()
        .map(|group_name| group_name.value().to_string())
        .collect();

    Ok(removed_group_names)
}
