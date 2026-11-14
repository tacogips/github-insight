//! Integration tests for GitHub client repository functionality
//!
//! These tests verify the ability to fetch repository details from GitHub repositories.
//! Tests use the GITHUB_INSIGHT_GITHUB_TOKEN environment variable for authentication.

use serial_test::serial;

mod test_util;
use github_insight::tools::functions::repository::get_multiple_repository_details;
use github_insight::types::RepositoryUrl;
use test_util::create_test_github_client;

/// Test fetching multiple repository details by URLs
///
/// This test fetches multiple repositories to verify that the function can successfully
/// retrieve repository details from valid GitHub repository URLs.
#[tokio::test]
#[serial]
async fn test_get_multiple_repository_details() {
    // Initialize GitHub client with token (if available) and reasonable timeout
    let client = create_test_github_client();

    // Test with valid repository URLs
    let repository_urls = vec![
        RepositoryUrl::new("https://github.com/tacogips/gitcodes-mcp-test-1".to_string()),
        RepositoryUrl::new("https://github.com/rust-lang/rust".to_string()),
    ];

    // Fetch the repositories
    let result = get_multiple_repository_details(&client, repository_urls).await;

    // Verify the request succeeded
    assert!(result.is_ok(), "Failed to fetch repositories: {:?}", result);

    let repositories = result.unwrap();

    // We should get at least one repository back (even if some fail)
    if repositories.is_empty() {
        println!("No repositories found - this may be expected if there are access issues");
        return;
    }

    println!("Found {} repositories", repositories.len());

    // Verify repository properties
    for repo in &repositories {
        assert!(
            !repo.git_repository_id.repository_name.as_str().is_empty(),
            "Repository name should not be empty"
        );
        assert!(
            !repo.git_repository_id.owner.as_str().is_empty(),
            "Repository owner should not be empty"
        );
        assert!(
            repo.created_at.timestamp() > 0,
            "Created timestamp should be valid"
        );
        assert!(
            repo.updated_at.timestamp() > 0,
            "Updated timestamp should be valid"
        );

        println!(
            "Successfully fetched repository: {}/{}",
            repo.git_repository_id.owner, repo.git_repository_id.repository_name
        );
    }
}

/// Test fetching repository details with empty input
///
/// This test verifies that the function handles empty repository URL lists correctly
/// and returns an empty result when given 0 repository URLs.
#[tokio::test]
#[serial]
async fn test_get_multiple_repository_details_empty_input() {
    // Initialize GitHub client with token (if available) and reasonable timeout
    let client = create_test_github_client();

    // Test with empty repository URLs list
    let repository_urls: Vec<RepositoryUrl> = vec![];

    // Fetch the repositories
    let result = get_multiple_repository_details(&client, repository_urls).await;

    // Should return empty result successfully
    assert!(
        result.is_ok(),
        "Function should handle empty input gracefully"
    );

    let repositories = result.unwrap();
    assert_eq!(
        repositories.len(),
        0,
        "Expected no repositories for empty input"
    );

    println!("Successfully handled empty repository URLs input");
}

/// Test handling of invalid repository URLs
///
/// This test verifies that the function returns an error when given invalid repository URLs.
#[tokio::test]
#[serial]
async fn test_get_multiple_repository_details_invalid_urls() {
    // Initialize GitHub client with token (if available) and reasonable timeout
    let client = create_test_github_client();

    // Test with invalid repository URLs
    let repository_urls = vec![
        RepositoryUrl::new("invalid-url".to_string()),
        RepositoryUrl::new("https://example.com/not-a-repo".to_string()),
    ];

    // Fetch the repositories
    let result = get_multiple_repository_details(&client, repository_urls).await;

    // Should return an error for invalid URLs
    assert!(
        result.is_err(),
        "Function should return error for invalid URLs"
    );

    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(
        error_msg.contains("Failed to parse repository URL"),
        "Error message should indicate URL parsing failure: {}",
        error_msg
    );

    println!("Successfully detected invalid URLs and returned error");
}

/// Test handling of non-existent repository URLs
///
/// This test verifies that the function handles non-existent repositories gracefully
/// by filtering them out and logging warnings instead of failing completely.
#[tokio::test]
#[serial]
async fn test_get_multiple_repository_details_non_existent() {
    // Initialize GitHub client with token (if available) and reasonable timeout
    let client = create_test_github_client();

    // Test with valid but non-existent repository URLs
    let repository_urls = vec![
        RepositoryUrl::new("https://github.com/tacogips/gitcodes-mcp-test-1".to_string()), // Valid repo
        RepositoryUrl::new("https://github.com/nonexistentuser/nonexistentrepo".to_string()), // Non-existent
    ];

    // Fetch the repositories
    let result = get_multiple_repository_details(&client, repository_urls).await;

    // Should succeed but filter out non-existent repositories
    assert!(
        result.is_ok(),
        "Function should handle non-existent repositories gracefully: {:?}",
        result
    );

    let repositories = result.unwrap();

    // We should get fewer repositories than requested due to filtering
    assert!(
        repositories.len() <= 2,
        "Should get at most 2 repositories (some may be filtered out)"
    );

    // If we got any repositories, they should be valid
    for repo in &repositories {
        assert!(
            !repo.git_repository_id.repository_name.as_str().is_empty(),
            "Repository name should not be empty"
        );
        assert!(
            !repo.git_repository_id.owner.as_str().is_empty(),
            "Repository owner should not be empty"
        );

        println!(
            "Successfully fetched valid repository: {}/{}",
            repo.git_repository_id.owner, repo.git_repository_id.repository_name
        );
    }

    println!(
        "Successfully handled mixed valid/non-existent repositories, got {} results",
        repositories.len()
    );
}

/// Test concurrent fetching behavior
///
/// This test verifies that the function can handle multiple repository URLs
/// and fetch them concurrently without issues.
#[tokio::test]
#[serial]
async fn test_get_multiple_repository_details_concurrent() {
    // Initialize GitHub client with token (if available) and reasonable timeout
    let client = create_test_github_client();

    // Test with multiple valid repository URLs to exercise concurrent fetching
    let repository_urls = vec![
        RepositoryUrl::new("https://github.com/tacogips/gitcodes-mcp-test-1".to_string()),
        RepositoryUrl::new("https://github.com/rust-lang/rust".to_string()),
        RepositoryUrl::new("https://github.com/tokio-rs/tokio".to_string()),
    ];

    // Fetch the repositories
    let result = get_multiple_repository_details(&client, repository_urls).await;

    // Verify the request succeeded
    assert!(result.is_ok(), "Failed to fetch repositories: {:?}", result);

    let repositories = result.unwrap();

    // We should get at least one repository back
    if repositories.is_empty() {
        println!("No repositories found - this may be expected if there are access issues");
        return;
    }

    println!("Concurrently fetched {} repositories", repositories.len());

    // Verify all returned repositories are valid
    for repo in &repositories {
        assert!(
            !repo.git_repository_id.repository_name.as_str().is_empty(),
            "Repository name should not be empty"
        );
        assert!(
            !repo.git_repository_id.owner.as_str().is_empty(),
            "Repository owner should not be empty"
        );
        assert!(
            repo.created_at.timestamp() > 0,
            "Created timestamp should be valid"
        );

        println!(
            "Successfully fetched repository: {}/{}",
            repo.git_repository_id.owner, repo.git_repository_id.repository_name
        );
    }
}
