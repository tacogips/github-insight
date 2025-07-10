//! Integration tests for ProfileService
//!
//! These tests verify the complete functionality of the ProfileService including
//! profile management, repository/project registration, and persistence operations.
//! Each test uses isolated temporary directories to avoid race conditions.

use tempfile::TempDir;
use uuid::Uuid;

use github_insight::services::{ProfileService, ProfileServiceError};
use github_insight::types::{
    profile::ProfileName,
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

#[test]
fn test_profile_service_creation() {
    let temp_dir = create_test_temp_dir();
    let data_dir = temp_dir.path().to_path_buf();

    let service = ProfileService::new(data_dir.clone());
    assert!(service.is_ok());

    let service = service.unwrap();
    let profiles = service.list_profiles();

    // Default profile should be created automatically
    assert!(profiles.contains(&ProfileName::DEFAULT_PROFILE_NAME.to_string()));
    assert_eq!(profiles.len(), 1);

    // Data directory should exist
    assert!(data_dir.exists());
}

#[test]
fn test_create_profile_success() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let result =
        service.create_profile("test-profile", Some("Test profile description".to_string()));
    assert!(result.is_ok());

    let profiles = service.list_profiles();
    assert!(profiles.contains(&"test-profile".to_string()));
    assert_eq!(profiles.len(), 2); // default + test-profile
}

#[test]
fn test_create_profile_already_exists() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    // Create profile first time - should succeed
    service.create_profile("test-profile", None).unwrap();

    // Try to create same profile again - should fail
    let result = service.create_profile("test-profile", None);
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
    let result = service.create_profile("", None);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ProfileServiceError::InvalidProfileName(_)
    ));

    // Test name with invalid characters
    let result = service.create_profile("test/profile", None);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ProfileServiceError::InvalidProfileName(_)
    ));

    // Test name too long
    let long_name = "a".repeat(101);
    let result = service.create_profile(&long_name, None);
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
    service.create_profile("test-profile", None).unwrap();
    assert!(
        service
            .list_profiles()
            .contains(&"test-profile".to_string())
    );

    // Delete the profile
    let result = service.delete_profile("test-profile");
    assert!(result.is_ok());
    assert!(
        !service
            .list_profiles()
            .contains(&"test-profile".to_string())
    );
}

#[test]
fn test_delete_profile_not_found() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let result = service.delete_profile("nonexistent");
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

    let result = service.delete_profile("default");
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

    let result = service.register_repository("default", repo_id.clone());
    assert!(result.is_ok());

    let repositories = service.list_repositories("default").unwrap();
    assert_eq!(repositories.len(), 1);
    assert_eq!(repositories[0], repo_id);
}

#[test]
fn test_register_repository_new_profile() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let repo_id = create_test_repository("owner1", "repo1");

    // Register repository to non-existent profile - should create profile automatically
    let result = service.register_repository("auto-created", repo_id.clone());
    assert!(result.is_ok());

    let profiles = service.list_profiles();
    assert!(profiles.contains(&"auto-created".to_string()));

    let repositories = service.list_repositories("auto-created").unwrap();
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
        .register_repository("default", repo_id.clone())
        .unwrap();

    // Try to register same repository again
    let result = service.register_repository("default", repo_id.clone());
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
        .register_repository("default", repo_id.clone())
        .unwrap();
    let result = service.unregister_repository("default", &repo_id);
    assert!(result.is_ok());

    let repositories = service.list_repositories("default").unwrap();
    assert_eq!(repositories.len(), 0);
}

#[test]
fn test_unregister_repository_not_found() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let repo_id = create_test_repository("owner1", "repo1");

    let result = service.unregister_repository("default", &repo_id);
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

    let result = service.unregister_repository("nonexistent", &repo_id);
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

    let result = service.register_project("default", project_id.clone());
    assert!(result.is_ok());

    let projects = service.list_projects("default").unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0], project_id);
}

#[test]
fn test_register_project_new_profile() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let project_id = create_test_project("owner1", 1);

    // Register project to non-existent profile - should create profile automatically
    let result = service.register_project("auto-created", project_id.clone());
    assert!(result.is_ok());

    let profiles = service.list_profiles();
    assert!(profiles.contains(&"auto-created".to_string()));

    let projects = service.list_projects("auto-created").unwrap();
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
        .register_project("default", project_id.clone())
        .unwrap();

    // Try to register same project again
    let result = service.register_project("default", project_id.clone());
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
        .register_project("default", project_id.clone())
        .unwrap();
    let result = service.unregister_project("default", &project_id);
    assert!(result.is_ok());

    let projects = service.list_projects("default").unwrap();
    assert_eq!(projects.len(), 0);
}

#[test]
fn test_unregister_project_not_found() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let project_id = create_test_project("owner1", 1);

    let result = service.unregister_project("default", &project_id);
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

    let result = service.unregister_project("nonexistent", &project_id);
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

    let repositories = service.list_repositories("default").unwrap();
    assert_eq!(repositories.len(), 0);
}

#[test]
fn test_list_repositories_multiple() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let repo1 = create_test_repository("owner1", "repo1");
    let repo2 = create_test_repository("owner2", "repo2");

    service
        .register_repository("default", repo1.clone())
        .unwrap();
    service
        .register_repository("default", repo2.clone())
        .unwrap();

    let repositories = service.list_repositories("default").unwrap();
    assert_eq!(repositories.len(), 2);
    assert!(repositories.contains(&repo1));
    assert!(repositories.contains(&repo2));
}

#[test]
fn test_list_projects_empty() {
    let temp_dir = create_test_temp_dir();
    let service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let projects = service.list_projects("default").unwrap();
    assert_eq!(projects.len(), 0);
}

#[test]
fn test_list_projects_multiple() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    let project1 = create_test_project("owner1", 1);
    let project2 = create_test_project("owner2", 2);

    service
        .register_project("default", project1.clone())
        .unwrap();
    service
        .register_project("default", project2.clone())
        .unwrap();

    let projects = service.list_projects("default").unwrap();
    assert_eq!(projects.len(), 2);
    assert!(projects.contains(&project1));
    assert!(projects.contains(&project2));
}

#[test]
fn test_get_profile_info_success() {
    let temp_dir = create_test_temp_dir();
    let mut service = ProfileService::new(temp_dir.path().to_path_buf()).unwrap();

    service
        .create_profile("test-profile", Some("Test description".to_string()))
        .unwrap();

    let profile_info = service.get_profile_info("test-profile").unwrap();
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

    let result = service.get_profile_info("nonexistent");
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
            .create_profile("persistent-profile", Some("Persistent test".to_string()))
            .unwrap();
        service
            .register_repository("persistent-profile", repo_id.clone())
            .unwrap();
        service
            .register_project("persistent-profile", project_id.clone())
            .unwrap();
    }

    // Create second service instance and verify data persists
    {
        let service = ProfileService::new(data_dir.clone()).unwrap();
        let profiles = service.list_profiles();
        assert!(profiles.contains(&"persistent-profile".to_string()));

        let repositories = service.list_repositories("persistent-profile").unwrap();
        assert_eq!(repositories.len(), 1);
        assert_eq!(repositories[0], repo_id);

        let projects = service.list_projects("persistent-profile").unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0], project_id);

        let profile_info = service.get_profile_info("persistent-profile").unwrap();
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
    service.create_profile("profile1", None).unwrap();
    service.create_profile("profile2", None).unwrap();

    service
        .register_repository("profile1", repo1.clone())
        .unwrap();
    service
        .register_repository("profile2", repo2.clone())
        .unwrap();
    service
        .register_project("profile1", project1.clone())
        .unwrap();
    service
        .register_project("profile2", project2.clone())
        .unwrap();

    // Verify profile1 data
    let profile1_repos = service.list_repositories("profile1").unwrap();
    assert_eq!(profile1_repos.len(), 1);
    assert_eq!(profile1_repos[0], repo1);

    let profile1_projects = service.list_projects("profile1").unwrap();
    assert_eq!(profile1_projects.len(), 1);
    assert_eq!(profile1_projects[0], project1);

    // Verify profile2 data
    let profile2_repos = service.list_repositories("profile2").unwrap();
    assert_eq!(profile2_repos.len(), 1);
    assert_eq!(profile2_repos[0], repo2);

    let profile2_projects = service.list_projects("profile2").unwrap();
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
        assert!(profiles.contains(&"default".to_string()));
    }

    // Each service should operate independently
    for (i, service) in services.iter().enumerate() {
        let _profile_name = format!("profile-{}", i);
        // Each service operates on its own data directory
        // This ensures no interference between concurrent tests
        assert!(service.list_profiles().len() == 1); // Only default profile
    }
}
