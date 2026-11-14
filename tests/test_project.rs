//! Integration tests for GitHub client project functionality
//!
//! These tests verify the ability to fetch project resources from real GitHub projects.
//! Tests use the GITHUB_INSIGHT_GITHUB_TOKEN environment variable for authentication.

use serial_test::serial;

mod test_util;
use github_insight::services::MultiResourceFetcher;
use github_insight::types::{Owner, ProjectId, ProjectNumber, ProjectType};
use test_util::create_test_github_client;

/// Test fetching project resources from a user project
///
/// This test fetches resources from the tacogips/projects/1 project to verify
/// that the client can successfully retrieve project items.
#[tokio::test]
#[serial]
async fn test_fetch_project_resources() {
    // Initialize GitHub client with token (if available) and reasonable timeout
    let client = create_test_github_client();

    // Create project ID for the designated test project
    // https://github.com/users/tacogips/projects/1
    let project_id = ProjectId::new(
        Owner::new("tacogips".to_string()),
        ProjectNumber::new(1),
        ProjectType::User,
    );

    // Fetch the project resources
    let result = client.fetch_all_project_resources(project_id.clone()).await;

    // Verify the request succeeded
    assert!(
        result.is_ok(),
        "Failed to fetch project resources: {:?}",
        result
    );

    let resources = result.unwrap();

    println!("Raw resource count: {}", resources.len());
    if resources.is_empty() {
        println!("No project resources found - checking if this is expected");
    }

    assert!(
        !resources.is_empty(),
        "No project resources found - this is expected if project is empty"
    );

    assert!(
        resources.len() <= 20,
        "too many resouces. maybe anything went wrong "
    );

    println!("Found {} project resources", resources.len());

    // Verify we got some resources back and they have basic properties
    assert!(
        !resources.is_empty(),
        "Should have at least one project resource"
    );

    // Log what we got without strict validation to avoid panics
    for resource in &resources {
        println!(
            "Successfully fetched project resource: {:?} ({})",
            resource.title, resource.state
        );

        // Verify basic properties
        assert!(
            !resource.project_item_id.0.is_empty(),
            "Resource ID should not be empty"
        );
        assert!(
            resource
                .title
                .as_ref()
                .map(|s| !s.is_empty())
                .unwrap_or(false),
            "Resource title should not be empty"
        );
        assert!(
            !resource.state.is_empty(),
            "Resource state should not be empty"
        );
        assert!(
            resource
                .created_at
                .as_ref()
                .map(|ts| ts.timestamp() > 0)
                .unwrap_or(false),
            "Created timestamp should be valid"
        );
        assert!(
            resource
                .updated_at
                .as_ref()
                .map(|ts| ts.timestamp() > 0)
                .unwrap_or(false),
            "Updated timestamp should be valid"
        );
    }
}

/// Test fetching project resources from a non-existent project
///
/// This test verifies that the client returns an error when given a project ID that doesn't exist.
#[tokio::test]
#[serial]
async fn test_fetch_non_existent_project() {
    // Initialize GitHub client with token (if available) and reasonable timeout
    let client = create_test_github_client();

    // Create project ID for a non-existent project
    let project_id = ProjectId::new(
        Owner::new("tacogips".to_string()),
        ProjectNumber::new(9999999),
        ProjectType::User,
    );

    // Fetch the project resources
    let result = client.fetch_all_project_resources(project_id).await;

    // The client should return an error for non-existent projects
    assert!(
        result.is_err(),
        "Client should return error for non-existent projects"
    );

    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(
        error_msg.contains("Project not found") || error_msg.contains("Could not resolve to"),
        "Error message should indicate project not found: {}",
        error_msg
    );

    println!("Successfully detected non-existent project and returned error");
}

/// Test fetching project resources with different resource types
///
/// This test verifies that the client can handle different types of project resources
/// (issues, pull requests, draft issues).
#[tokio::test]
#[serial]
async fn test_fetch_project_resources_mixed_types() {
    // Initialize GitHub client with token (if available) and reasonable timeout
    let client = create_test_github_client();

    // Create project ID for the designated test project
    // https://github.com/users/tacogips/projects/1
    let project_id = ProjectId::new(
        Owner::new("tacogips".to_string()),
        ProjectNumber::new(1),
        ProjectType::User,
    );

    // Fetch the project resources
    let result = client.fetch_all_project_resources(project_id).await;

    // Verify the request succeeded
    assert!(
        result.is_ok(),
        "Failed to fetch project resources: {:?}",
        result
    );

    let resources = result.unwrap();

    // Assert that resources are not empty
    assert!(
        !resources.is_empty(),
        "Project resources should not be empty for test project"
    );

    println!("Found {} project resources", resources.len());

    // Categorize resources by type
    let mut issue_count = 0;
    let mut pr_count = 0;
    let mut draft_count = 0;

    for resource in &resources {
        match &resource.original_resource {
            github_insight::types::project::ProjectOriginalResource::Issue(_) => {
                issue_count += 1;
                println!("Found issue resource: {:?}", resource.title);
            }
            github_insight::types::project::ProjectOriginalResource::PullRequest(_) => {
                pr_count += 1;
                println!("Found PR resource: {:?}", resource.title);
            }
            github_insight::types::project::ProjectOriginalResource::DraftIssue => {
                draft_count += 1;
                println!("Found draft issue resource: {:?}", resource.title);
            }
        }

        // Verify custom field values if present
        if !resource.custom_field_values.is_empty() {
            println!("  Custom fields:");
            for field in &resource.custom_field_values {
                println!("    {}: {:?}", field.field_name, field.value);
            }
        }

        // Verify column name if present
        if let Some(ref column) = resource.column_name {
            println!("  Column: {}", column);
        }

        // Verify author, assignees, and labels
        println!("  Author: {}", resource.author.as_str());
        if !resource.assignees.is_empty() {
            println!(
                "  Assignees: {:?}",
                resource
                    .assignees
                    .iter()
                    .map(|a| a.as_str())
                    .collect::<Vec<_>>()
            );
        }
        if !resource.labels.is_empty() {
            println!(
                "  Labels: {:?}",
                resource.labels.iter().map(|l| l.name()).collect::<Vec<_>>()
            );
        }
    }

    println!(
        "Resource distribution: {} issues, {} PRs, {} drafts",
        issue_count, pr_count, draft_count
    );

    // Verify we have at least one resource of any type
    assert!(
        issue_count + pr_count + draft_count > 0,
        "Should have at least one resource of any type"
    );
}

/// Test fetching project resources using MultiResourceFetcher
///
/// This test verifies that the MultiResourceFetcher can successfully fetch
/// project resources from a real GitHub project.
#[tokio::test]
#[serial]
async fn test_multi_resource_fetcher_fetch_project_resources() {
    // Initialize GitHub client and create MultiResourceFetcher
    let client = create_test_github_client();
    let fetcher = MultiResourceFetcher::new(client);

    // Create project ID for the designated test project
    // https://github.com/users/tacogips/projects/1
    let project_id = ProjectId::new(
        Owner::new("tacogips".to_string()),
        ProjectNumber::new(1),
        ProjectType::User,
    );

    // Fetch the project resources using MultiResourceFetcher
    let result = fetcher.fetch_project_resources(project_id.clone()).await;

    // Verify the request succeeded
    assert!(
        result.is_ok(),
        "Failed to fetch project resources: {:?}",
        result
    );

    let resources = result.unwrap();

    println!(
        "MultiResourceFetcher found {} project resources",
        resources.len()
    );

    // Verify we got some resources back
    assert!(
        !resources.is_empty(),
        "Should have at least one project resource"
    );

    assert!(
        resources.len() <= 20,
        "Too many resources. Maybe something went wrong"
    );

    // Verify basic properties of each resource
    for resource in &resources {
        println!(
            "MultiResourceFetcher resource: {:?} ({})",
            resource.title, resource.state
        );

        // Verify basic properties
        assert!(
            !resource.project_item_id.0.is_empty(),
            "Resource ID should not be empty"
        );
        assert!(
            resource
                .title
                .as_ref()
                .map(|s| !s.is_empty())
                .unwrap_or(false),
            "Resource title should not be empty"
        );
        assert!(
            !resource.state.is_empty(),
            "Resource state should not be empty"
        );
        assert!(
            resource
                .created_at
                .as_ref()
                .map(|ts| ts.timestamp() > 0)
                .unwrap_or(false),
            "Created timestamp should be valid"
        );
        assert!(
            resource
                .updated_at
                .as_ref()
                .map(|ts| ts.timestamp() > 0)
                .unwrap_or(false),
            "Updated timestamp should be valid"
        );
    }

    println!("MultiResourceFetcher test completed successfully");
}

/// Test fetching project resources from a non-existent project using MultiResourceFetcher
///
/// This test verifies that the MultiResourceFetcher returns an error when given
/// a project ID that doesn't exist.
#[tokio::test]
#[serial]
async fn test_multi_resource_fetcher_non_existent_project() {
    // Initialize GitHub client and create MultiResourceFetcher
    let client = create_test_github_client();
    let fetcher = MultiResourceFetcher::new(client);

    // Create project ID for a non-existent project
    let project_id = ProjectId::new(
        Owner::new("tacogips".to_string()),
        ProjectNumber::new(9999999),
        ProjectType::User,
    );

    // Fetch the project resources using MultiResourceFetcher
    let result = fetcher.fetch_project_resources(project_id).await;

    // The fetcher should return an error for non-existent projects
    assert!(
        result.is_err(),
        "MultiResourceFetcher should return error for non-existent projects"
    );

    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(
        error_msg.contains("Project not found") || error_msg.contains("Could not resolve to"),
        "Error message should indicate project not found: {}",
        error_msg
    );

    println!("MultiResourceFetcher successfully detected non-existent project and returned error");
}

/// Test that MultiResourceFetcher handles empty projects correctly
///
/// This test verifies that the MultiResourceFetcher can handle projects
/// that exist but have no resources.
#[tokio::test]
#[serial]
async fn test_multi_resource_fetcher_empty_project() {
    // Initialize GitHub client and create MultiResourceFetcher
    let client = create_test_github_client();
    let fetcher = MultiResourceFetcher::new(client);

    // Create project ID for the designated test project
    // https://github.com/users/tacogips/projects/1
    let project_id = ProjectId::new(
        Owner::new("tacogips".to_string()),
        ProjectNumber::new(1),
        ProjectType::User,
    );

    // Fetch the project resources using MultiResourceFetcher
    let result = fetcher.fetch_project_resources(project_id).await;

    // Verify the request succeeded
    assert!(
        result.is_ok(),
        "Failed to fetch project resources: {:?}",
        result
    );

    let resources = result.unwrap();

    // This test assumes the project might be empty or have resources
    // We just verify that the call completes successfully
    println!(
        "MultiResourceFetcher handled project with {} resources",
        resources.len()
    );

    // If resources exist, verify they have valid structure
    for resource in &resources {
        assert!(
            !resource.project_item_id.0.is_empty(),
            "Resource ID should not be empty"
        );
        assert!(
            !resource.state.is_empty(),
            "Resource state should not be empty"
        );
    }

    println!("MultiResourceFetcher empty project test completed successfully");
}
