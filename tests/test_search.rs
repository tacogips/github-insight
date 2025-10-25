//! Integration tests for GitHub search functionality
//!
//! These tests verify the ability to search for issues and pull requests from real GitHub repositories.
//! Tests use the GITHUB_INSIGHT_GITHUB_TOKEN environment variable for authentication.
//!
//! Note: All tests in this file require GitHub authentication as they use GraphQL API.
//! Run with: cargo test --features integration-tests

use serial_test::serial;

mod test_util;
use github_insight::types::{IssueOrPullrequest, RepositoryId, SearchCursor, SearchQuery};
use test_util::create_test_github_client;

/// Test basic search functionality with general query
///
/// This test verifies that the search_resources function can successfully
/// search for issues and pull requests using a general query.
#[tokio::test]
#[serial]
#[cfg(feature = "integration-tests")]
async fn test_search_resources_basic() {
    // Initialize GitHub client with token (if available) and reasonable timeout
    let client = create_test_github_client();

    // Create repository ID for a well-known repository
    let repository_id =
        RepositoryId::new("tacogips".to_string(), "gitcodes-mcp-test-1".to_string());

    // Search for issues/PRs with "test" keyword
    let query = SearchQuery::new("test".to_string());

    // Fetch the search results
    let result = client
        .search_resources(
            repository_id.clone(),
            query,
            Some(10), // Limit to 10 results
            None,     // No cursor
        )
        .await;

    // Verify the result is successful
    assert!(
        result.is_ok(),
        "Search should be successful: {:?}",
        result.err()
    );

    let search_result = result.unwrap();
    let search_results = search_result.issue_or_pull_requests;

    // Verify we got valid results
    assert!(
        search_results.len() <= 10,
        "Should not exceed requested limit of 10 results"
    );

    // IMPORTANT: Verify that we actually found some results to validate the search is working
    assert!(
        !search_results.is_empty(),
        "Search should return at least some results for 'test' query in test repository. \
        If this fails, the repository may be empty or the search isn't working properly."
    );

    // Verify each result is a valid IssueOrPullrequest
    for result in &search_results {
        match result {
            IssueOrPullrequest::Issue(issue) => {
                assert!(!issue.title.is_empty(), "Issue title should not be empty");
                assert!(issue.issue_id.number > 0, "Issue number should be positive");
                // Verify the search query worked by checking if "test" appears in title or would be found
                println!("Found issue #{}: {}", issue.issue_id.number, issue.title);
            }
            IssueOrPullrequest::PullRequest(pr) => {
                assert!(!pr.title.is_empty(), "PR title should not be empty");
                assert!(
                    pr.pull_request_id.number > 0,
                    "PR number should be positive"
                );
                println!("Found PR #{}: {}", pr.pull_request_id.number, pr.title);
            }
        }
    }

    println!(
        "Successfully searched and found {} results",
        search_results.len()
    );
}

/// Test search with specific query terms
///
/// This test verifies that the search_resources function can handle
/// more complex queries with specific terms.
#[tokio::test]
#[serial]
#[cfg(feature = "integration-tests")]
async fn test_search_resources_with_specific_query() {
    // Initialize GitHub client with token (if available) and reasonable timeout
    let client = create_test_github_client();

    // Create repository ID for the test repository
    let repository_id =
        RepositoryId::new("tacogips".to_string(), "gitcodes-mcp-test-1".to_string());

    // Search for issues/PRs with "bug" label
    let query = SearchQuery::new("label:bug".to_string());

    // Fetch the search results
    let result = client
        .search_resources(
            repository_id.clone(),
            query,
            Some(5), // Limit to 5 results
            None,    // No cursor
        )
        .await;

    // Verify the result is successful
    assert!(
        result.is_ok(),
        "Search with label filter should be successful: {:?}",
        result.err()
    );

    let search_result = result.unwrap();
    let search_results = search_result.issue_or_pull_requests;

    // Verify we got valid results (may be empty if no bugs exist)
    assert!(
        search_results.len() <= 5,
        "Should not exceed requested limit of 5 results"
    );

    println!(
        "Successfully searched with label filter and found {} results",
        search_results.len()
    );
}

/// Test search with empty query
///
/// This test verifies that the search_resources function can handle
/// an empty query gracefully.
#[tokio::test]
#[serial]
#[cfg(feature = "integration-tests")]
async fn test_search_resources_empty_query() {
    // Initialize GitHub client with token (if available) and reasonable timeout
    let client = create_test_github_client();

    // Create repository ID for the test repository
    let repository_id =
        RepositoryId::new("tacogips".to_string(), "gitcodes-mcp-test-1".to_string());

    // Search with empty query (should return all issues/PRs)
    let query = SearchQuery::new("".to_string());

    // Fetch the search results
    let result = client
        .search_resources(
            repository_id.clone(),
            query,
            Some(3), // Limit to 3 results
            None,    // No cursor
        )
        .await;

    // Verify the result is successful
    assert!(
        result.is_ok(),
        "Search with empty query should be successful: {:?}",
        result.err()
    );

    let search_result = result.unwrap();
    let search_results = search_result.issue_or_pull_requests;

    // Verify we got valid results
    assert!(
        search_results.len() <= 3,
        "Should not exceed requested limit of 3 results"
    );

    // For empty query, we expect to get some results (all issues/PRs in the repo)
    assert!(
        !search_results.is_empty(),
        "Empty query search should return at least some results from the repository. \
        If this fails, the repository may be completely empty."
    );

    // Verify each result is valid
    for result in &search_results {
        match result {
            IssueOrPullrequest::Issue(issue) => {
                assert!(!issue.title.is_empty(), "Issue title should not be empty");
                assert!(issue.issue_id.number > 0, "Issue number should be positive");
                println!("Found issue #{}: {}", issue.issue_id.number, issue.title);
            }
            IssueOrPullrequest::PullRequest(pr) => {
                assert!(!pr.title.is_empty(), "PR title should not be empty");
                assert!(
                    pr.pull_request_id.number > 0,
                    "PR number should be positive"
                );
                println!("Found PR #{}: {}", pr.pull_request_id.number, pr.title);
            }
        }
    }

    println!(
        "Successfully searched with empty query and found {} results",
        search_results.len()
    );
}

/// Test search with pagination cursor
///
/// This test verifies that the search_resources function can handle
/// pagination using a cursor (though we can't easily test the actual cursor logic
/// without a real multi-page result set).
#[tokio::test]
#[serial]
#[cfg(feature = "integration-tests")]
async fn test_search_resources_with_pagination() {
    // Initialize GitHub client with token (if available) and reasonable timeout
    let client = create_test_github_client();

    // Create repository ID for the test repository
    let repository_id =
        RepositoryId::new("tacogips".to_string(), "gitcodes-mcp-test-1".to_string());

    // Search with small page size to potentially trigger pagination
    let query = SearchQuery::new("is:issue".to_string());

    // First page
    let result_page1 = client
        .search_resources(
            repository_id.clone(),
            query.clone(),
            Some(1), // Very small page size
            None,    // No cursor for first page
        )
        .await;

    // Verify the first page is successful
    assert!(
        result_page1.is_ok(),
        "First page search should be successful: {:?}",
        result_page1.err()
    );

    let search_result_page1 = result_page1.unwrap();
    let search_results_page1 = search_result_page1.issue_or_pull_requests;

    // Verify we got valid results
    assert!(
        search_results_page1.len() <= 1,
        "Should not exceed requested limit of 1 result"
    );

    // Verify that we found at least one issue to test pagination properly
    assert!(
        !search_results_page1.is_empty(),
        "First page should return at least one issue for 'is:issue' query. \
        If this fails, there may be no issues in the repository."
    );

    // Verify the result is actually an issue
    for result in &search_results_page1 {
        match result {
            IssueOrPullrequest::Issue(issue) => {
                assert!(!issue.title.is_empty(), "Issue title should not be empty");
                assert!(issue.issue_id.number > 0, "Issue number should be positive");
                println!("Found issue #{}: {}", issue.issue_id.number, issue.title);
            }
            IssueOrPullrequest::PullRequest(_) => {
                panic!("Expected issue but got pull request for 'is:issue' query");
            }
        }
    }

    // Test with a dummy cursor (this won't work in practice but tests the interface)
    let dummy_cursor = SearchCursor("dummy_cursor".to_string());
    let result_with_cursor = client
        .search_resources(
            repository_id.clone(),
            query,
            Some(1),
            Some(dummy_cursor), // Dummy cursor
        )
        .await;

    // This may fail due to invalid cursor, but the function should handle it gracefully
    println!(
        "Pagination test completed. First page had {} results",
        search_results_page1.len()
    );
    if let Err(e) = result_with_cursor {
        println!("Cursor test failed as expected with dummy cursor: {}", e);
    }
}

/// Test search with non-existent repository
///
/// This test verifies that the search_resources function handles
/// searches in non-existent repositories gracefully.
#[tokio::test]
#[serial]
#[cfg(feature = "integration-tests")]
async fn test_search_resources_nonexistent_repo() {
    // Initialize GitHub client with token (if available) and reasonable timeout
    let client = create_test_github_client();

    // Create repository ID for a non-existent repository
    let repository_id = RepositoryId::new(
        "nonexistent-user".to_string(),
        "nonexistent-repo".to_string(),
    );

    // Search for issues/PRs
    let query = SearchQuery::new("test".to_string());

    // Fetch the search results
    let result = client
        .search_resources(repository_id.clone(), query, Some(10), None)
        .await;

    // The search should either succeed with empty results or fail gracefully
    match result {
        Ok(search_result) => {
            // Should return empty results for non-existent repo
            assert_eq!(
                search_result.issue_or_pull_requests.len(),
                0,
                "Non-existent repo should return empty results"
            );
            println!("Non-existent repo search returned empty results as expected");
        }
        Err(e) => {
            // Should return a meaningful error
            let error_msg = e.to_string();
            println!("Non-existent repo search failed as expected: {}", error_msg);
        }
    }
}

/// Test search filtering for pull requests only
///
/// This test verifies that the search_resources function can filter
/// search results to return only pull requests using the "is:pr" query.
#[tokio::test]
#[serial]
#[cfg(feature = "integration-tests")]
async fn test_search_resources_pull_requests_only() {
    // Initialize GitHub client with token (if available) and reasonable timeout
    let client = create_test_github_client();

    // Create repository ID for the test repository
    let repository_id =
        RepositoryId::new("tacogips".to_string(), "gitcodes-mcp-test-1".to_string());

    // Search for pull requests only
    let query = SearchQuery::new("is:pr".to_string());

    // Fetch the search results
    let result = client
        .search_resources(
            repository_id.clone(),
            query,
            Some(10), // Limit to 10 results
            None,     // No cursor
        )
        .await;

    // Verify the result is successful
    assert!(
        result.is_ok(),
        "Search for PRs only should be successful: {:?}",
        result.err()
    );

    let search_result = result.unwrap();
    let search_results = search_result.issue_or_pull_requests;

    // Verify we got valid results
    assert!(
        search_results.len() <= 10,
        "Should not exceed requested limit of 10 results"
    );

    // Verify that all results are pull requests
    for result in &search_results {
        match result {
            IssueOrPullrequest::Issue(_) => {
                panic!("Expected only pull requests but found an issue for 'is:pr' query");
            }
            IssueOrPullrequest::PullRequest(pr) => {
                assert!(!pr.title.is_empty(), "PR title should not be empty");
                assert!(
                    pr.pull_request_id.number > 0,
                    "PR number should be positive"
                );
                println!("Found PR #{}: {}", pr.pull_request_id.number, pr.title);
            }
        }
    }

    println!(
        "Successfully searched for PRs only and found {} results",
        search_results.len()
    );
}

/// Test search filtering for issues only
///
/// This test verifies that the search_resources function can filter
/// search results to return only issues using the "is:issue" query.
#[tokio::test]
#[serial]
#[cfg(feature = "integration-tests")]
async fn test_search_resources_issues_only() {
    // Initialize GitHub client with token (if available) and reasonable timeout
    let client = create_test_github_client();

    // Create repository ID for the test repository
    let repository_id =
        RepositoryId::new("tacogips".to_string(), "gitcodes-mcp-test-1".to_string());

    // Search for issues only
    let query = SearchQuery::new("is:issue".to_string());

    // Fetch the search results
    let result = client
        .search_resources(
            repository_id.clone(),
            query,
            Some(10), // Limit to 10 results
            None,     // No cursor
        )
        .await;

    // Verify the result is successful
    assert!(
        result.is_ok(),
        "Search for issues only should be successful: {:?}",
        result.err()
    );

    let search_result = result.unwrap();
    let search_results = search_result.issue_or_pull_requests;

    // Verify we got valid results
    assert!(
        search_results.len() <= 10,
        "Should not exceed requested limit of 10 results"
    );

    // Verify that all results are issues
    for result in &search_results {
        match result {
            IssueOrPullrequest::Issue(issue) => {
                assert!(!issue.title.is_empty(), "Issue title should not be empty");
                assert!(issue.issue_id.number > 0, "Issue number should be positive");
                println!("Found issue #{}: {}", issue.issue_id.number, issue.title);
            }
            IssueOrPullrequest::PullRequest(_) => {
                panic!("Expected only issues but found a pull request for 'is:issue' query");
            }
        }
    }

    println!(
        "Successfully searched for issues only and found {} results",
        search_results.len()
    );
}

/// Test search filtering for both issues and pull requests
///
/// This test verifies that the search_resources function can return
/// both issues and pull requests when no type filter is applied.
#[tokio::test]
#[serial]
#[cfg(feature = "integration-tests")]
async fn test_search_resources_both_types() {
    // Initialize GitHub client with token (if available) and reasonable timeout
    let client = create_test_github_client();

    // Create repository ID for the test repository
    let repository_id =
        RepositoryId::new("tacogips".to_string(), "gitcodes-mcp-test-1".to_string());

    // Search for both issues and PRs (no type filter)
    let query = SearchQuery::new("state:open OR state:closed".to_string());

    // Fetch the search results
    let result = client
        .search_resources(
            repository_id.clone(),
            query,
            Some(20), // Larger limit to get mix of both types
            None,     // No cursor
        )
        .await;

    // Verify the result is successful
    assert!(
        result.is_ok(),
        "Search for both types should be successful: {:?}",
        result.err()
    );

    let search_result = result.unwrap();
    let search_results = search_result.issue_or_pull_requests;

    // Verify we got valid results
    assert!(
        search_results.len() <= 20,
        "Should not exceed requested limit of 20 results"
    );

    // Count issues and pull requests
    let mut issue_count = 0;
    let mut pr_count = 0;

    for result in &search_results {
        match result {
            IssueOrPullrequest::Issue(issue) => {
                assert!(!issue.title.is_empty(), "Issue title should not be empty");
                assert!(issue.issue_id.number > 0, "Issue number should be positive");
                issue_count += 1;
                println!("Found issue #{}: {}", issue.issue_id.number, issue.title);
            }
            IssueOrPullrequest::PullRequest(pr) => {
                assert!(!pr.title.is_empty(), "PR title should not be empty");
                assert!(
                    pr.pull_request_id.number > 0,
                    "PR number should be positive"
                );
                pr_count += 1;
                println!("Found PR #{}: {}", pr.pull_request_id.number, pr.title);
            }
        }
    }

    println!(
        "Successfully searched for both types and found {} issues and {} PRs (total: {})",
        issue_count,
        pr_count,
        search_results.len()
    );

    // Note: We don't assert that both types must be present since the repository
    // might only have one type, but we verify that the search doesn't filter incorrectly
}

/// Test search with pagination to verify next_pager is Some
///
/// This test verifies that the search_resources function returns a SearchResult
/// with next_pager as Some when there are more results available.
#[tokio::test]
#[serial]
#[cfg(feature = "integration-tests")]
async fn test_search_resources_next_pager_some() {
    // Initialize GitHub client with token (if available) and reasonable timeout
    let client = create_test_github_client();

    // Create repository ID for the test repository
    let repository_id =
        RepositoryId::new("tacogips".to_string(), "gitcodes-mcp-test-1".to_string());

    // Search with a very small limit to likely trigger pagination
    let query = SearchQuery::new("".to_string()); // Empty query to get all results

    // Fetch the search results with a very small limit to force pagination
    let result = client
        .search_resources(
            repository_id.clone(),
            query,
            Some(1), // Very small limit to ensure pagination
            None,    // No cursor
        )
        .await;

    // Verify the result is successful
    assert!(
        result.is_ok(),
        "Search should be successful: {:?}",
        result.err()
    );

    let search_result = result.unwrap();

    // Verify that we got exactly 1 result (the limit we set)
    assert_eq!(
        search_result.issue_or_pull_requests.len(),
        1,
        "Should return exactly 1 result due to limit"
    );

    // Verify that next_pager is Some, indicating there are more results
    assert!(
        search_result.next_pager.is_some(),
        "next_pager should be Some when there are more results available with small limit"
    );

    // Verify the next_pager structure
    let next_pager = search_result.next_pager.unwrap();
    assert!(
        next_pager.has_next_page,
        "has_next_page should be true when there are more results"
    );

    // next_page_cursor should be Some when has_next_page is true
    assert!(
        next_pager.next_page_cursor.is_some(),
        "next_page_cursor should be Some when has_next_page is true"
    );

    println!(
        "Successfully verified next_pager is Some with has_next_page={} and cursor present",
        next_pager.has_next_page
    );
}

/// Test search pagination by fetching next page with different results
///
/// This test verifies that the search_resources function can use pagination
/// to fetch the next page and returns different results from the first page.
#[tokio::test]
#[serial]
#[cfg(feature = "integration-tests")]
async fn test_search_resources_pagination_next_page() {
    // Initialize GitHub client with token (if available) and reasonable timeout
    let client = create_test_github_client();

    // Create repository ID for the test repository
    let repository_id =
        RepositoryId::new("tacogips".to_string(), "gitcodes-mcp-test-1".to_string());

    // Search with empty query to get all results
    let query = SearchQuery::new("".to_string());

    // First page - get only 1 result to force pagination
    let result_page1 = client
        .search_resources(
            repository_id.clone(),
            query.clone(),
            Some(1), // Very small limit to ensure pagination
            None,    // No cursor for first page
        )
        .await;

    // Verify the first page is successful
    assert!(
        result_page1.is_ok(),
        "First page search should be successful: {:?}",
        result_page1.err()
    );

    let search_result_page1 = result_page1.unwrap();

    // Verify that we got exactly 1 result
    assert_eq!(
        search_result_page1.issue_or_pull_requests.len(),
        1,
        "First page should return exactly 1 result due to limit"
    );

    // Verify that next_pager is Some
    assert!(
        search_result_page1.next_pager.is_some(),
        "next_pager should be Some on first page when there are more results"
    );

    let next_pager = search_result_page1.next_pager.unwrap();
    assert!(
        next_pager.has_next_page,
        "has_next_page should be true on first page"
    );

    // Get the cursor for the next page
    let next_cursor = next_pager
        .next_page_cursor
        .expect("next_page_cursor should be Some");

    // Store the first page result for comparison
    let first_page_result = &search_result_page1.issue_or_pull_requests[0];
    let first_page_id = match first_page_result {
        IssueOrPullrequest::Issue(issue) => issue.issue_id.number,
        IssueOrPullrequest::PullRequest(pr) => pr.pull_request_id.number,
    };

    // Second page - use the cursor from the first page
    let result_page2 = client
        .search_resources(
            repository_id.clone(),
            query.clone(),
            Some(1),           // Same small limit
            Some(next_cursor), // Use cursor from first page
        )
        .await;

    // Verify the second page is successful
    assert!(
        result_page2.is_ok(),
        "Second page search should be successful: {:?}",
        result_page2.err()
    );

    let search_result_page2 = result_page2.unwrap();

    // Verify that we got exactly 1 result on second page
    assert_eq!(
        search_result_page2.issue_or_pull_requests.len(),
        1,
        "Second page should return exactly 1 result due to limit"
    );

    // Verify that the second page result is different from the first page
    let second_page_result = &search_result_page2.issue_or_pull_requests[0];
    let second_page_id = match second_page_result {
        IssueOrPullrequest::Issue(issue) => issue.issue_id.number,
        IssueOrPullrequest::PullRequest(pr) => pr.pull_request_id.number,
    };

    // Assert that the results are different
    assert_ne!(
        first_page_id, second_page_id,
        "Second page should return different results from first page. First: {}, Second: {}",
        first_page_id, second_page_id
    );

    // Verify that both results are valid
    match first_page_result {
        IssueOrPullrequest::Issue(issue) => {
            assert!(
                !issue.title.is_empty(),
                "First page issue title should not be empty"
            );
            assert!(
                issue.issue_id.number > 0,
                "First page issue number should be positive"
            );
            println!(
                "First page - Found issue #{}: {}",
                issue.issue_id.number, issue.title
            );
        }
        IssueOrPullrequest::PullRequest(pr) => {
            assert!(
                !pr.title.is_empty(),
                "First page PR title should not be empty"
            );
            assert!(
                pr.pull_request_id.number > 0,
                "First page PR number should be positive"
            );
            println!(
                "First page - Found PR #{}: {}",
                pr.pull_request_id.number, pr.title
            );
        }
    }

    match second_page_result {
        IssueOrPullrequest::Issue(issue) => {
            assert!(
                !issue.title.is_empty(),
                "Second page issue title should not be empty"
            );
            assert!(
                issue.issue_id.number > 0,
                "Second page issue number should be positive"
            );
            println!(
                "Second page - Found issue #{}: {}",
                issue.issue_id.number, issue.title
            );
        }
        IssueOrPullrequest::PullRequest(pr) => {
            assert!(
                !pr.title.is_empty(),
                "Second page PR title should not be empty"
            );
            assert!(
                pr.pull_request_id.number > 0,
                "Second page PR number should be positive"
            );
            println!(
                "Second page - Found PR #{}: {}",
                pr.pull_request_id.number, pr.title
            );
        }
    }

    println!(
        "Successfully verified pagination: First page had item #{}, Second page had item #{} (different results)",
        first_page_id, second_page_id
    );
}

/// Test repository ID overrides repo parameter in query
///
/// This test verifies that when a query contains a repo:owner/name pattern,
/// it gets overwritten by the repository_id parameter in search_resources.
#[tokio::test]
#[serial]
#[cfg(feature = "integration-tests")]
async fn test_search_resources_repo_override() {
    // Initialize GitHub client with token (if available) and reasonable timeout
    let client = create_test_github_client();

    // Create repository ID for the test repository
    let repository_id =
        RepositoryId::new("tacogips".to_string(), "gitcodes-mcp-test-1".to_string());

    // Search query with different repo parameter that should be overridden
    let query = SearchQuery::new("repo:microsoft/vscode test".to_string());

    // Fetch the search results
    let result = client
        .search_resources(
            repository_id.clone(),
            query,
            Some(5), // Limit to 5 results
            None,    // No cursor
        )
        .await;

    // Verify the result is successful
    assert!(
        result.is_ok(),
        "Search with repo override should be successful: {:?}",
        result.err()
    );

    let search_result = result.unwrap();
    let search_results = search_result.issue_or_pull_requests;

    // Verify we got valid results (may be empty if no matches in target repo)
    assert!(
        search_results.len() <= 5,
        "Should not exceed requested limit of 5 results"
    );

    // Verify all results are from the correct repository (tacogips/gitcodes-mcp-test-1)
    // not from the repo specified in the query (microsoft/vscode)
    for result in &search_results {
        match result {
            IssueOrPullrequest::Issue(issue) => {
                assert!(!issue.title.is_empty(), "Issue title should not be empty");
                assert!(issue.issue_id.number > 0, "Issue number should be positive");
                // The repository information should match our repository_id, not the query repo
                println!(
                    "Found issue #{}: {} in repo {}/{}",
                    issue.issue_id.number,
                    issue.title,
                    issue.issue_id.git_repository.owner,
                    issue.issue_id.git_repository.repository_name
                );
                assert_eq!(
                    issue.issue_id.git_repository.owner, repository_id.owner,
                    "Issue should be from the repository_id owner, not query repo"
                );
                assert_eq!(
                    issue.issue_id.git_repository.repository_name, repository_id.repository_name,
                    "Issue should be from the repository_id repo, not query repo"
                );
            }
            IssueOrPullrequest::PullRequest(pr) => {
                assert!(!pr.title.is_empty(), "PR title should not be empty");
                assert!(
                    pr.pull_request_id.number > 0,
                    "PR number should be positive"
                );
                println!(
                    "Found PR #{}: {} in repo {}/{}",
                    pr.pull_request_id.number,
                    pr.title,
                    pr.pull_request_id.git_repository.owner,
                    pr.pull_request_id.git_repository.repository_name
                );
                assert_eq!(
                    pr.pull_request_id.git_repository.owner, repository_id.owner,
                    "PR should be from the repository_id owner, not query repo"
                );
                assert_eq!(
                    pr.pull_request_id.git_repository.repository_name,
                    repository_id.repository_name,
                    "PR should be from the repository_id repo, not query repo"
                );
            }
        }
    }

    println!(
        "Successfully verified repo override: found {} results from {} instead of repo specified in query",
        search_results.len(),
        format!("{}/{}", repository_id.owner, repository_id.repository_name)
    );
}
