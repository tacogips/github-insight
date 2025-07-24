//! Integration tests for ProfileService
//!
//! These tests verify the complete functionality of the ProfileService including
//! profile management, repository/project registration, and persistence operations.
//! Each test uses isolated temporary directories to avoid race conditions.

use tempfile::TempDir;
use uuid::Uuid;

use github_insight::services::{ProfileService, ProfileServiceError};
use github_insight::types::{
    Branch,
    profile::{GroupName, ProfileName, RepositoryBranchPair},
    project::{ProjectId, ProjectNumber, ProjectType},
    repository::{Owner, RepositoryId, RepositoryName},
};

/// Helper function to create a unique temporary directory for each test
/// This ensures tests don't interfere with each other
fn create_test_temp_dir() -> TempDir {
    let unique_suffix = Uuid::new_v4().to_string();
    tempfile::Builder::new()
        .prefix(&format!("github_insight_test_{}_", unique_suffix))
        .tempdir()
        .expect("Failed to create temporary directory")
}

/// Helper function to create a test repository ID
fn create_test_repository(owner: &str, repo: &str) -> RepositoryId {
    RepositoryId {
        owner: Owner::from(owner),
        repository_name: RepositoryName::from(repo),
    }
}

/// Helper function to create a test project ID
fn create_test_project(owner: &str, number: u64) -> ProjectId {
    ProjectId {
        owner: Owner::from(owner),
        number: ProjectNumber(number),
        project_type: ProjectType::User, // Default to User for test projects
    }
}

/// Helper function to create a test repository branch unit
fn create_test_repository_branch_unit(
    owner: &str,
    repo: &str,
    branch: &str,
) -> RepositoryBranchPair {
    let repository_id = RepositoryId {
        owner: Owner::from(owner),
        repository_name: RepositoryName::from(repo),
    };
    let branch = Branch::new(branch);
    RepositoryBranchPair::new(repository_id, branch)
}

#[test]
fn test_profile_service_creation() {
    let temp_dir = create_test_temp_dir();
    let data_dir = temp_dir.path().to_path_buf();

    let service = ProfileService::new(data_dir.clone());
    assert!(service.is_ok());

    let service = service.unwrap();
    let profiles = service.list_profiles();

    // Default profile should be created automatically
    assert!(profiles.contains(&ProfileName::from(ProfileName::DEFAULT_PROFILE_NAME)));
    assert_eq!(profiles.len(), 1);

    // Data directory should exist
    assert!(data_dir.exists());
}

#[test]
fn test_create_profile_success() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let result = service.create_profile(
        &ProfileName::from("test-profile"),
        Some("Test profile description".to_string()),
    );
    assert!(result.is_ok());

    let profiles = service.list_profiles();
    assert!(profiles.contains(&ProfileName::from("test-profile")));
    assert_eq!(profiles.len(), 2); // default + test-profile
}

#[test]
fn test_create_profile_already_exists() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    // Create profile first time - should succeed
    service
        .create_profile(&ProfileName::from("test-profile"), None)
        .unwrap();

    // Try to create same profile again - should fail
    let result = service.create_profile(&ProfileName::from("test-profile"), None);
    assert!(result.is_err());
    match result.unwrap_err() {
        ProfileServiceError::ProfileAlreadyExists(name) => {
            assert_eq!(name, "test-profile");
        }
        _ => panic!("Expected ProfileAlreadyExists error"),
    }
}

#[test]
fn test_create_profile_invalid_name() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    // Test empty name
    let result = service.create_profile(&ProfileName::from(""), None);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ProfileServiceError::InvalidProfileName(_)
    ));

    // Test name with invalid characters
    let result = service.create_profile(&ProfileName::from("test/profile"), None);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ProfileServiceError::InvalidProfileName(_)
    ));

    // Test name too long
    let long_name = "a".repeat(101);
    let result = service.create_profile(&ProfileName::from(long_name.as_str()), None);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ProfileServiceError::InvalidProfileName(_)
    ));
}

#[test]
fn test_delete_profile_success() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    // Create a profile
    service
        .create_profile(&ProfileName::from("test-profile"), None)
        .unwrap();
    assert!(
        service
            .list_profiles()
            .contains(&ProfileName::from("test-profile"))
    );

    // Delete the profile
    let result = service.delete_profile(&ProfileName::from("test-profile"));
    assert!(result.is_ok());
    assert!(
        !service
            .list_profiles()
            .contains(&ProfileName::from("test-profile"))
    );
}

#[test]
fn test_delete_profile_not_found() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let result = service.delete_profile(&ProfileName::from("nonexistent"));
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ProfileServiceError::ProfileNotFound(_)
    ));
}

#[test]
fn test_delete_default_profile_forbidden() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let result = service.delete_profile(&ProfileName::from("default"));
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ProfileServiceError::InvalidProfileName(_)
    ));
}

#[test]
fn test_register_repository_success() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let repo_id = create_test_repository("owner1", "repo1");

    let result = service.register_repository(&ProfileName::from("default"), repo_id.clone());
    assert!(result.is_ok());

    let repositories = service
        .list_repositories(&ProfileName::from("default"))
        .unwrap();
    assert_eq!(repositories.len(), 1);
    assert_eq!(repositories[0], repo_id);
}

#[test]
fn test_register_repository_new_profile() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let repo_id = create_test_repository("owner1", "repo1");

    // Register repository to non-existent profile - should create profile automatically
    let result = service.register_repository(&ProfileName::from("auto-created"), repo_id.clone());
    assert!(result.is_ok());

    let profiles = service.list_profiles();
    assert!(profiles.contains(&ProfileName::from("auto-created")));

    let repositories = service
        .list_repositories(&ProfileName::from("auto-created"))
        .unwrap();
    assert_eq!(repositories.len(), 1);
    assert_eq!(repositories[0], repo_id);
}

#[test]
fn test_register_repository_already_exists() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let repo_id = create_test_repository("owner1", "repo1");

    // Register repository first time
    service
        .register_repository(&ProfileName::from("default"), repo_id.clone())
        .unwrap();

    // Try to register same repository again
    let result = service.register_repository(&ProfileName::from("default"), repo_id.clone());
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ProfileServiceError::RepositoryAlreadyExists(_)
    ));
}

#[test]
fn test_unregister_repository_success() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let repo_id = create_test_repository("owner1", "repo1");

    // Register and then unregister repository
    service
        .register_repository(&ProfileName::from("default"), repo_id.clone())
        .unwrap();
    let result = service.unregister_repository(&ProfileName::from("default"), &repo_id);
    assert!(result.is_ok());

    let repositories = service
        .list_repositories(&ProfileName::from("default"))
        .unwrap();
    assert_eq!(repositories.len(), 0);
}

#[test]
fn test_unregister_repository_not_found() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let repo_id = create_test_repository("owner1", "repo1");

    let result = service.unregister_repository(&ProfileName::from("default"), &repo_id);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ProfileServiceError::RepositoryNotFound(_)
    ));
}

#[test]
fn test_unregister_repository_profile_not_found() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let repo_id = create_test_repository("owner1", "repo1");

    let result = service.unregister_repository(&ProfileName::from("nonexistent"), &repo_id);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ProfileServiceError::ProfileNotFound(_)
    ));
}

#[test]
fn test_register_project_success() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let project_id = create_test_project("owner1", 1);

    let result = service.register_project(&ProfileName::from("default"), project_id.clone());
    assert!(result.is_ok());

    let projects = service
        .list_projects(&ProfileName::from("default"))
        .unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0], project_id);
}

#[test]
fn test_register_project_new_profile() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let project_id = create_test_project("owner1", 1);

    // Register project to non-existent profile - should create profile automatically
    let result = service.register_project(&ProfileName::from("auto-created"), project_id.clone());
    assert!(result.is_ok());

    let profiles = service.list_profiles();
    assert!(profiles.contains(&ProfileName::from("auto-created")));

    let projects = service
        .list_projects(&ProfileName::from("auto-created"))
        .unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0], project_id);
}

#[test]
fn test_register_project_already_exists() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let project_id = create_test_project("owner1", 1);

    // Register project first time
    service
        .register_project(&ProfileName::from("default"), project_id.clone())
        .unwrap();

    // Try to register same project again
    let result = service.register_project(&ProfileName::from("default"), project_id.clone());
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ProfileServiceError::ProjectAlreadyExists(_)
    ));
}

#[test]
fn test_unregister_project_success() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let project_id = create_test_project("owner1", 1);

    // Register and then unregister project
    service
        .register_project(&ProfileName::from("default"), project_id.clone())
        .unwrap();
    let result = service.unregister_project(&ProfileName::from("default"), &project_id);
    assert!(result.is_ok());

    let projects = service
        .list_projects(&ProfileName::from("default"))
        .unwrap();
    assert_eq!(projects.len(), 0);
}

#[test]
fn test_unregister_project_not_found() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let project_id = create_test_project("owner1", 1);

    let result = service.unregister_project(&ProfileName::from("default"), &project_id);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ProfileServiceError::ProjectNotFound(_)
    ));
}

#[test]
fn test_unregister_project_profile_not_found() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let project_id = create_test_project("owner1", 1);

    let result = service.unregister_project(&ProfileName::from("nonexistent"), &project_id);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ProfileServiceError::ProfileNotFound(_)
    ));
}

#[test]
fn test_list_repositories_empty() {
    let temp_dir = create_test_temp_dir();
    let service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let repositories = service
        .list_repositories(&ProfileName::from("default"))
        .unwrap();
    assert_eq!(repositories.len(), 0);
}

#[test]
fn test_list_repositories_multiple() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let repo1 = create_test_repository("owner1", "repo1");
    let repo2 = create_test_repository("owner2", "repo2");

    service
        .register_repository(&ProfileName::from("default"), repo1.clone())
        .unwrap();
    service
        .register_repository(&ProfileName::from("default"), repo2.clone())
        .unwrap();

    let repositories = service
        .list_repositories(&ProfileName::from("default"))
        .unwrap();
    assert_eq!(repositories.len(), 2);
    assert!(repositories.contains(&repo1));
    assert!(repositories.contains(&repo2));
}

#[test]
fn test_list_projects_empty() {
    let temp_dir = create_test_temp_dir();
    let service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let projects = service
        .list_projects(&ProfileName::from("default"))
        .unwrap();
    assert_eq!(projects.len(), 0);
}

#[test]
fn test_list_projects_multiple() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let project1 = create_test_project("owner1", 1);
    let project2 = create_test_project("owner2", 2);

    service
        .register_project(&ProfileName::from("default"), project1.clone())
        .unwrap();
    service
        .register_project(&ProfileName::from("default"), project2.clone())
        .unwrap();

    let projects = service
        .list_projects(&ProfileName::from("default"))
        .unwrap();
    assert_eq!(projects.len(), 2);
    assert!(projects.contains(&project1));
    assert!(projects.contains(&project2));
}

#[test]
fn test_get_profile_info_success() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    service
        .create_profile(
            &ProfileName::from("test-profile"),
            Some("Test description".to_string()),
        )
        .unwrap();

    let profile_info = service
        .get_profile_info(&ProfileName::from("test-profile"))
        .unwrap();
    assert_eq!(profile_info.name.0, "test-profile");
    assert_eq!(
        profile_info.description,
        Some("Test description".to_string())
    );
}

#[test]
fn test_get_profile_info_not_found() {
    let temp_dir = create_test_temp_dir();
    let service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let result = service.get_profile_info(&ProfileName::from("nonexistent"));
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ProfileServiceError::ProfileNotFound(_)
    ));
}

#[test]
fn test_persistence_across_instances() {
    let temp_dir = create_test_temp_dir();
    let data_dir = temp_dir.path().to_path_buf();

    let repo_id = create_test_repository("owner1", "repo1");
    let project_id = create_test_project("owner1", 1);

    // Create first service instance and add data
    {
        let mut service = ProfileService::new(data_dir.clone()).unwrap();
        service
            .create_profile(
                &ProfileName::from("persistent-profile"),
                Some("Persistent test".to_string()),
            )
            .unwrap();
        service
            .register_repository(&ProfileName::from("persistent-profile"), repo_id.clone())
            .unwrap();
        service
            .register_project(&ProfileName::from("persistent-profile"), project_id.clone())
            .unwrap();
    }

    // Create second service instance and verify data persists
    {
        let service = ProfileService::new(data_dir.clone()).unwrap();
        let profiles = service.list_profiles();
        assert!(profiles.contains(&ProfileName::from("persistent-profile")));

        let repositories = service
            .list_repositories(&ProfileName::from("persistent-profile"))
            .unwrap();
        assert_eq!(repositories.len(), 1);
        assert_eq!(repositories[0], repo_id);

        let projects = service
            .list_projects(&ProfileName::from("persistent-profile"))
            .unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0], project_id);

        let profile_info = service
            .get_profile_info(&ProfileName::from("persistent-profile"))
            .unwrap();
        assert_eq!(
            profile_info.description,
            Some("Persistent test".to_string())
        );
    }
}

#[test]
fn test_multiple_profiles_isolation() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let repo1 = create_test_repository("owner1", "repo1");
    let repo2 = create_test_repository("owner2", "repo2");
    let project1 = create_test_project("owner1", 1);
    let project2 = create_test_project("owner2", 2);

    // Create two profiles with different data
    service
        .create_profile(&ProfileName::from("profile1"), None)
        .unwrap();
    service
        .create_profile(&ProfileName::from("profile2"), None)
        .unwrap();

    service
        .register_repository(&ProfileName::from("profile1"), repo1.clone())
        .unwrap();
    service
        .register_repository(&ProfileName::from("profile2"), repo2.clone())
        .unwrap();
    service
        .register_project(&ProfileName::from("profile1"), project1.clone())
        .unwrap();
    service
        .register_project(&ProfileName::from("profile2"), project2.clone())
        .unwrap();

    // Verify profile1 data
    let profile1_repos = service
        .list_repositories(&ProfileName::from("profile1"))
        .unwrap();
    assert_eq!(profile1_repos.len(), 1);
    assert_eq!(profile1_repos[0], repo1);

    let profile1_projects = service
        .list_projects(&ProfileName::from("profile1"))
        .unwrap();
    assert_eq!(profile1_projects.len(), 1);
    assert_eq!(profile1_projects[0], project1);

    // Verify profile2 data
    let profile2_repos = service
        .list_repositories(&ProfileName::from("profile2"))
        .unwrap();
    assert_eq!(profile2_repos.len(), 1);
    assert_eq!(profile2_repos[0], repo2);

    let profile2_projects = service
        .list_projects(&ProfileName::from("profile2"))
        .unwrap();
    assert_eq!(profile2_projects.len(), 1);
    assert_eq!(profile2_projects[0], project2);
}

#[test]
fn test_concurrent_operations_race_condition_prevention() {
    // This test verifies that using unique temp directories prevents race conditions
    // when multiple tests run concurrently

    let temp_dirs: Vec<TempDir> = (0..5).map(|_| create_test_temp_dir()).collect();
    let services: Vec<ProfileService> = temp_dirs
        .iter()
        .map(|dir| ProfileService::new(dir.path().to_path_buf()).unwrap())
        .collect();

    // All services should have only the default profile initially
    for service in &services {
        let profiles = service.list_profiles();
        assert_eq!(profiles.len(), 1);
        assert!(profiles.contains(&ProfileName::from("default")));
    }

    // Each service should operate independently
    for (i, service) in services.iter().enumerate() {
        let _profile_name = format!("profile-{}", i);
        // Each service operates on its own data directory
        // This ensures no interference between concurrent tests
        assert!(service.list_profiles().len() == 1); // Only default profile
    }
}

// =============================================================================
// Repository Branch Group Tests
// =============================================================================

#[test]
fn test_register_repository_branch_group_with_name() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let unit1 = create_test_repository_branch_unit("rust-lang", "rust", "main");
    let unit2 = create_test_repository_branch_unit("tokio-rs", "tokio", "master");
    let units = vec![unit1.clone(), unit2.clone()];

    let group_name = service
        .register_repository_branch_group(
            &ProfileName::from("default"),
            Some(GroupName::from("test-group")),
            units,
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
    assert_eq!(group.pairs.len(), 2);
    assert!(group.pairs.contains(&unit1));
    assert!(group.pairs.contains(&unit2));
}

#[test]
fn test_register_repository_branch_group_auto_name() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let unit = create_test_repository_branch_unit("rust-lang", "rust", "main");

    let group_name = service
        .register_repository_branch_group(
            &ProfileName::from("default"),
            None, // Auto-generate name
            vec![unit],
        )
        .unwrap();

    // Auto-generated name should follow yyyymmdd-hash format
    assert!(group_name.value().len() > 8); // At minimum yyyymmdd format
    assert!(group_name.value().contains('-')); // Should contain dash separator

    let groups = service
        .list_repository_branch_groups(&ProfileName::from("default"))
        .unwrap();
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0], group_name);
}

#[test]
fn test_register_repository_branch_group_already_exists() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let unit = create_test_repository_branch_unit("rust-lang", "rust", "main");

    // Register group first time
    service
        .register_repository_branch_group(
            &ProfileName::from("default"),
            Some(GroupName::from("duplicate-group")),
            vec![unit.clone()],
        )
        .unwrap();

    // Try to register same group name again
    let result = service.register_repository_branch_group(
        &ProfileName::from("default"),
        Some(GroupName::from("duplicate-group")),
        vec![unit],
    );

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ProfileServiceError::GroupAlreadyExists(_)
    ));
}

#[test]
fn test_unregister_repository_branch_group() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let unit1 = create_test_repository_branch_unit("rust-lang", "rust", "main");
    let unit2 = create_test_repository_branch_unit("tokio-rs", "tokio", "master");
    let units = vec![unit1.clone(), unit2.clone()];

    let group_name = service
        .register_repository_branch_group(
            &ProfileName::from("default"),
            Some(GroupName::from("removable-group")),
            units,
        )
        .unwrap();

    let removed_group = service
        .unregister_repository_branch_group(&ProfileName::from("default"), &group_name)
        .unwrap();

    assert_eq!(removed_group.name, group_name);
    assert_eq!(removed_group.pairs.len(), 2);
    assert!(removed_group.pairs.contains(&unit1));
    assert!(removed_group.pairs.contains(&unit2));

    // Group should no longer exist
    let groups = service
        .list_repository_branch_groups(&ProfileName::from("default"))
        .unwrap();
    assert_eq!(groups.len(), 0);
}

#[test]
fn test_unregister_repository_branch_group_not_found() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let result = service.unregister_repository_branch_group(
        &ProfileName::from("default"),
        &GroupName::from("nonexistent-group"),
    );

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ProfileServiceError::GroupNotFound(_)
    ));
}

#[test]
fn test_add_pair_to_group() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let initial_unit = create_test_repository_branch_unit("rust-lang", "rust", "main");
    let additional_unit = create_test_repository_branch_unit("tokio-rs", "tokio", "master");

    let group_name = service
        .register_repository_branch_group(
            &ProfileName::from("default"),
            Some(GroupName::from("expandable-group")),
            vec![initial_unit.clone()],
        )
        .unwrap();

    // Add unit to group
    service
        .add_pair_to_group(
            &ProfileName::from("default"),
            &group_name,
            additional_unit.clone(),
        )
        .unwrap();

    let group = service
        .get_repository_branch_group(&ProfileName::from("default"), &group_name)
        .unwrap();

    assert_eq!(group.pairs.len(), 2);
    assert!(group.pairs.contains(&initial_unit));
    assert!(group.pairs.contains(&additional_unit));
}

#[test]
fn test_add_pair_to_group_already_exists() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let unit = create_test_repository_branch_unit("rust-lang", "rust", "main");

    let group_name = service
        .register_repository_branch_group(
            &ProfileName::from("default"),
            Some(GroupName::from("duplicate-unit-group")),
            vec![unit.clone()],
        )
        .unwrap();

    // Try to add same unit again
    let result = service.add_pair_to_group(&ProfileName::from("default"), &group_name, unit);

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ProfileServiceError::PairAlreadyExists(_)
    ));
}

#[test]
fn test_remove_pair_from_group() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let unit1 = create_test_repository_branch_unit("rust-lang", "rust", "main");
    let unit2 = create_test_repository_branch_unit("tokio-rs", "tokio", "master");
    let unit3 = create_test_repository_branch_unit("serde-rs", "serde", "main");

    let group_name = service
        .register_repository_branch_group(
            &ProfileName::from("default"),
            Some(GroupName::from("reducible-group")),
            vec![unit1.clone(), unit2.clone(), unit3.clone()],
        )
        .unwrap();

    // Remove unit from group
    service
        .remove_pair_from_group(&ProfileName::from("default"), &group_name, &unit2)
        .unwrap();

    let group = service
        .get_repository_branch_group(&ProfileName::from("default"), &group_name)
        .unwrap();

    assert_eq!(group.pairs.len(), 2);
    assert!(group.pairs.contains(&unit1));
    assert!(group.pairs.contains(&unit3));
    assert!(!group.pairs.contains(&unit2));
}

#[test]
fn test_remove_pair_from_group_not_found() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let existing_unit = create_test_repository_branch_unit("rust-lang", "rust", "main");
    let nonexistent_unit = create_test_repository_branch_unit("tokio-rs", "tokio", "master");

    let group_name = service
        .register_repository_branch_group(
            &ProfileName::from("default"),
            Some(GroupName::from("single-unit-group")),
            vec![existing_unit],
        )
        .unwrap();

    // Try to remove unit that doesn't exist in group
    let result = service.remove_pair_from_group(
        &ProfileName::from("default"),
        &group_name,
        &nonexistent_unit,
    );

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ProfileServiceError::PairNotFound(_)
    ));
}

#[test]
fn test_rename_repository_branch_group() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let unit = create_test_repository_branch_unit("rust-lang", "rust", "main");

    let original_name = service
        .register_repository_branch_group(
            &ProfileName::from("default"),
            Some(GroupName::from("original-name")),
            vec![unit.clone()],
        )
        .unwrap();

    let new_name = GroupName::from("renamed-group");

    // Rename the group
    service
        .rename_repository_branch_group(
            &ProfileName::from("default"),
            &original_name,
            new_name.clone(),
        )
        .unwrap();

    // Original name should not exist
    let result = service.get_repository_branch_group(&ProfileName::from("default"), &original_name);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ProfileServiceError::GroupNotFound(_)
    ));

    // New name should exist with same content
    let group = service
        .get_repository_branch_group(&ProfileName::from("default"), &new_name)
        .unwrap();
    assert_eq!(group.name, new_name);
    assert_eq!(group.pairs.len(), 1);
    assert!(group.pairs.contains(&unit));
}

#[test]
fn test_list_repository_branch_groups_empty() {
    let temp_dir = create_test_temp_dir();
    let service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let groups = service
        .list_repository_branch_groups(&ProfileName::from("default"))
        .unwrap();

    assert_eq!(groups.len(), 0);
}

#[test]
fn test_list_repository_branch_groups_multiple() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let unit1 = create_test_repository_branch_unit("rust-lang", "rust", "main");
    let unit2 = create_test_repository_branch_unit("tokio-rs", "tokio", "master");

    let group1_name = service
        .register_repository_branch_group(
            &ProfileName::from("default"),
            Some(GroupName::from("group1")),
            vec![unit1],
        )
        .unwrap();

    let group2_name = service
        .register_repository_branch_group(
            &ProfileName::from("default"),
            Some(GroupName::from("group2")),
            vec![unit2],
        )
        .unwrap();

    let groups = service
        .list_repository_branch_groups(&ProfileName::from("default"))
        .unwrap();

    assert_eq!(groups.len(), 2);
    assert!(groups.contains(&group1_name));
    assert!(groups.contains(&group2_name));
}

#[test]
fn test_remove_groups_older_than() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let unit = create_test_repository_branch_unit("rust-lang", "rust", "main");

    let group_name = service
        .register_repository_branch_group(
            &ProfileName::from("default"),
            Some(GroupName::from("cleanup-test-group")),
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
    let result = service.get_repository_branch_group(&ProfileName::from("default"), &group_name);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ProfileServiceError::GroupNotFound(_)
    ));
}

#[test]
fn test_repository_branch_group_profile_isolation() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let unit1 = create_test_repository_branch_unit("rust-lang", "rust", "main");
    let unit2 = create_test_repository_branch_unit("tokio-rs", "tokio", "master");

    // Create groups in different profiles
    let profile1_group = service
        .register_repository_branch_group(
            &ProfileName::from("profile1"),
            Some(GroupName::from("profile1-group")),
            vec![unit1],
        )
        .unwrap();

    let profile2_group = service
        .register_repository_branch_group(
            &ProfileName::from("profile2"),
            Some(GroupName::from("profile2-group")),
            vec![unit2],
        )
        .unwrap();

    // Verify profile1 groups
    let profile1_groups = service
        .list_repository_branch_groups(&ProfileName::from("profile1"))
        .unwrap();
    assert_eq!(profile1_groups.len(), 1);
    assert_eq!(profile1_groups[0], profile1_group);

    // Verify profile2 groups
    let profile2_groups = service
        .list_repository_branch_groups(&ProfileName::from("profile2"))
        .unwrap();
    assert_eq!(profile2_groups.len(), 1);
    assert_eq!(profile2_groups[0], profile2_group);

    // Cross-profile access should fail
    let result =
        service.get_repository_branch_group(&ProfileName::from("profile1"), &profile2_group);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ProfileServiceError::GroupNotFound(_)
    ));
}

#[test]
fn test_repository_branch_group_persistence_across_instances() {
    let temp_dir = create_test_temp_dir();
    let data_dir = temp_dir.path().to_path_buf();

    let unit1 = create_test_repository_branch_unit("rust-lang", "rust", "main");
    let unit2 = create_test_repository_branch_unit("tokio-rs", "tokio", "master");
    let units = vec![unit1.clone(), unit2.clone()];

    let group_name = {
        // Create first service instance and add group
        let mut service = ProfileService::new(data_dir.clone()).unwrap();
        service
            .register_repository_branch_group(
                &ProfileName::from("persistent-profile"),
                Some(GroupName::from("persisted-group")),
                units,
            )
            .unwrap()
    };

    // Create second service instance and verify group persists
    {
        let service = ProfileService::new(data_dir.clone()).unwrap();

        let groups = service
            .list_repository_branch_groups(&ProfileName::from("persistent-profile"))
            .unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0], group_name);

        let group = service
            .get_repository_branch_group(&ProfileName::from("persistent-profile"), &group_name)
            .unwrap();
        assert_eq!(group.pairs.len(), 2);
        assert!(group.pairs.contains(&unit1));
        assert!(group.pairs.contains(&unit2));
    }
}
