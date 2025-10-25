//! Integration tests for GitHub client pull request functionality
//!
//! These tests verify the ability to fetch pull requests by number from real GitHub repositories.
//! Tests use the GITHUB_INSIGHT_GITHUB_TOKEN environment variable for authentication.

use std::env;
use tokio::time::Duration;

use github_insight::github::client::GitHubClient;

/// Creates a configured GitHub client for testing
///
/// This function creates a GitHubClient instance with the GitHub token from environment
/// (if available) and a reasonable timeout for integration tests. If no token is available,
/// the client will work with public repositories only.
pub fn create_test_github_client() -> GitHubClient {
    let token = env::var("GITHUB_INSIGHT_GITHUB_TOKEN").ok();
    // Use shorter timeout for tests to avoid long delays
    GitHubClient::new(token, Some(Duration::from_secs(15)))
        .expect("Failed to create GitHub client for testing. Note: GraphQL API requires authentication even for public repositories.")
}
