//! Profile management tool functions
//!
//! This module provides MCP tool functions for profile management operations,
//! including creating, listing, and deleting profiles, as well as managing
//! repositories and projects within profiles.

use crate::services::{ProfileService, default_profile_config_dir};
use crate::types::profile::ProfileInfo;
use crate::types::{ProfileName, ProjectId, RepositoryId};

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
pub async fn list_repositories(profile_name: String) -> Result<Vec<RepositoryId>, String> {
    let config_dir = default_profile_config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;

    let service = ProfileService::new(config_dir)
        .map_err(|e| format!("Failed to create profile service: {}", e))?;

    let profile_name = ProfileName::from(profile_name.as_str());

    let repositories = service
        .list_repositories(&profile_name)
        .map_err(|e| format!("Failed to list repositories: {}", e))?;

    Ok(repositories)
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
pub async fn list_projects(profile_name: String) -> Result<Vec<ProjectId>, String> {
    let config_dir = default_profile_config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;

    let service = ProfileService::new(config_dir)
        .map_err(|e| format!("Failed to create profile service: {}", e))?;

    let profile_name = ProfileName::from(profile_name.as_str());

    let projects = service
        .list_projects(&profile_name)
        .map_err(|e| format!("Failed to list projects: {}", e))?;

    Ok(projects)
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
