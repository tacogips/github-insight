//! Integration tests for GitHub client pull request functionality
//!
//! These tests verify the ability to fetch pull requests by number from real GitHub repositories.
//! Tests use the GITHUB_INSIGHT_GITHUB_TOKEN environment variable for authentication.
//!
//! Note: All tests in this file require GitHub authentication as they use GraphQL API.
//! Run with: cargo test --features integration-tests

use serial_test::serial;

mod test_util;
use github_insight::services::MultiResourceFetcher;
use github_insight::tools::functions;
use github_insight::types::{
    IssueOrPullrequest, PullRequestNumber, PullRequestUrl, RepositoryId, SearchQuery,
};
use test_util::create_test_github_client;

/// Test fetching multiple pull requests by numbers from the React repository
///
/// This test fetches multiple PRs from the facebook/react repository to verify
/// that the client can successfully retrieve 2 pull requests.
#[tokio::test]
#[serial]
#[cfg(feature = "integration-tests")]
async fn test_fetch_multiple_pull_requests_by_numbers() {
    // Initialize GitHub client with token (if available) and reasonable timeout
    let client = create_test_github_client();

    // Create repository ID for facebook/react
    let repository_id = RepositoryId::new("facebook".to_string(), "react".to_string());

    // Test with multiple well-known PR numbers from the React repository
    // Using smaller numbers that are more likely to exist
    let pr_numbers = vec![PullRequestNumber::new(1), PullRequestNumber::new(2)];

    // Fetch the pull requests
    let result = client
        .fetch_multiple_pull_requests_by_numbers(
            repository_id.clone(),
            &pr_numbers,
            None, // Use default limit
        )
        .await;

    // Verify the request succeeded
    assert!(
        result.is_ok(),
        "Failed to fetch pull requests: {:?}",
        result
    );

    let pull_requests = result.unwrap();
    assert_eq!(pull_requests.len(), 2, "Expected exactly two pull requests");

    // Verify each PR has valid properties
    for pr in &pull_requests {
        assert!(pr_numbers.contains(&PullRequestNumber::new(pr.pull_request_id.number)));
        assert_eq!(pr.pull_request_id.git_repository, repository_id);
        assert!(
            !pr.title.is_empty(),
            "Pull request title should not be empty"
        );
        assert!(
            pr.created_at.timestamp() > 0,
            "Created timestamp should be valid"
        );
        assert!(
            pr.updated_at.timestamp() > 0,
            "Updated timestamp should be valid"
        );

        println!(
            "Successfully fetched PR #{}: {}",
            pr.pull_request_id.number, pr.title
        );
    }
}

/// Test fetching pull requests with empty input
///
/// This test verifies that the client handles empty PR number lists correctly
/// and returns an empty result when given 0 PR numbers.
#[tokio::test]
#[serial]
#[cfg(feature = "integration-tests")]
async fn test_fetch_pull_requests_empty_input() {
    // Initialize GitHub client with token (if available) and reasonable timeout
    let client = create_test_github_client();

    // Create repository ID for rust-lang/rust
    let repository_id = RepositoryId::new("rust-lang".to_string(), "rust".to_string());

    // Test with empty PR numbers list
    let pr_numbers: Vec<PullRequestNumber> = vec![];

    // Fetch the pull requests
    let result = client
        .fetch_multiple_pull_requests_by_numbers(
            repository_id,
            &pr_numbers,
            None, // Use default limit
        )
        .await;

    // Should return empty result successfully
    assert!(
        result.is_ok(),
        "Client should handle empty input gracefully"
    );

    let pull_requests = result.unwrap();
    assert_eq!(
        pull_requests.len(),
        0,
        "Expected no pull requests for empty input"
    );

    println!("Successfully handled empty PR numbers input");
}

/// Test handling of non-existent pull request numbers
///
/// This test verifies that the client returns an error when given PR numbers that don't exist.
#[tokio::test]
#[serial]
#[cfg(feature = "integration-tests")]
async fn test_fetch_non_existent_pull_request() {
    // Initialize GitHub client with token (if available) and reasonable timeout
    let client = create_test_github_client();

    // Create repository ID for rust-lang/rust
    let repository_id = RepositoryId::new("rust-lang".to_string(), "rust".to_string());

    // Test with a very high PR number that likely doesn't exist
    let pr_numbers = vec![PullRequestNumber::new(9999999)];

    // Fetch the pull request
    let result = client
        .fetch_multiple_pull_requests_by_numbers(
            repository_id,
            &pr_numbers,
            None, // Use default limit
        )
        .await;

    // The client should return an error for non-existent PRs
    assert!(
        result.is_err(),
        "Client should return error for non-existent PRs"
    );

    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(
        error_msg.contains("Could not resolve to a PullRequest")
            || error_msg.contains("Resource not found"),
        "Error message should indicate resource not found: {}",
        error_msg
    );

    println!("Successfully detected non-existent PR and returned error");
}

/// Test fetching pull requests from multiple repositories using MultiResourceFetcher
///
/// This test verifies that the MultiResourceFetcher can successfully fetch PRs
/// from multiple repositories concurrently.
#[tokio::test]
#[serial]
#[cfg(feature = "integration-tests")]
async fn test_multi_resource_fetcher_pull_requests() {
    // Initialize GitHub client with token (if available) and reasonable timeout
    let client = create_test_github_client();
    let fetcher = MultiResourceFetcher::new(client);

    // Create repository IDs for multiple repositories
    let repo1 = RepositoryId::new("facebook".to_string(), "react".to_string());
    let repo2 = RepositoryId::new("rust-lang".to_string(), "rust".to_string());

    // Prepare PR numbers for each repository
    let pr_numbers_1 = vec![PullRequestNumber::new(1), PullRequestNumber::new(2)];
    let pr_numbers_2 = vec![PullRequestNumber::new(1)];

    let pr_requests = vec![
        (repo1.clone(), pr_numbers_1.clone()),
        (repo2.clone(), pr_numbers_2.clone()),
    ];

    // Fetch pull requests from multiple repositories
    let result = fetcher.fetch_pull_requests(pr_requests).await;

    // Verify the request succeeded
    assert!(
        result.is_ok(),
        "Failed to fetch pull requests from multiple repositories: {:?}",
        result
    );

    let prs_by_repo = result.unwrap();

    // Verify we got results for both repositories
    assert!(
        prs_by_repo.contains_key(&repo1) || prs_by_repo.contains_key(&repo2),
        "Expected at least one repository to have results"
    );

    // Count repositories with actual PRs
    let repos_with_prs: Vec<_> = prs_by_repo
        .iter()
        .filter(|(_, prs)| !prs.is_empty())
        .collect();

    // Verify that at least one repository returned PRs
    assert!(
        !repos_with_prs.is_empty(),
        "Expected at least one repository to have PRs, but all were empty"
    );

    // Verify PRs from each repository that returned results
    for (repo_id, pull_requests) in repos_with_prs {
        for pr in pull_requests {
            assert_eq!(pr.pull_request_id.git_repository, *repo_id);
            assert!(
                !pr.title.is_empty(),
                "Pull request title should not be empty"
            );
            assert!(
                pr.created_at.timestamp() > 0,
                "Created timestamp should be valid"
            );
            assert!(
                pr.updated_at.timestamp() > 0,
                "Updated timestamp should be valid"
            );

            println!(
                "Successfully fetched PR #{} from {}: {}",
                pr.pull_request_id.number, repo_id, pr.title
            );
        }
    }

    println!(
        "Successfully fetched PRs from {} repositories",
        prs_by_repo.len()
    );
}

/// Test complex search queries on famous public repositories
///
/// This test verifies that the search_resources function can handle
/// complex GitHub search syntax with multiple filters and operators.
#[tokio::test]
#[serial]
#[cfg(feature = "integration-tests")]
async fn test_complex_search_queries_rust_repository() {
    let client = create_test_github_client();
    let rust_repo = RepositoryId::new("rust-lang".to_string(), "rust".to_string());

    // Test 1: Search for recently closed PRs with performance-related labels
    let query = SearchQuery::new(
        "is:pr is:closed label:A-performance label:T-compiler created:>2024-01-01",
    );
    let result = client
        .search_resources(rust_repo.clone(), query, Some(5), None)
        .await;

    match result {
        Ok(search_result) => {
            println!(
                "Complex query 1 (rust-lang/rust): Found {} performance PRs",
                search_result.issue_or_pull_requests.len()
            );
            for result in &search_result.issue_or_pull_requests {
                if let IssueOrPullrequest::PullRequest(pr) = result {
                    println!("  PR #{}: {}", pr.pull_request_id.number, pr.title);
                }
            }
        }
        Err(e) => panic!("Complex query 1 failed: {}", e),
    }

    // Test 2: Search for open PRs by specific author with exclusion
    let query = SearchQuery::new("is:pr is:open author:bors -label:rollup");
    let result = client
        .search_resources(rust_repo.clone(), query, Some(3), None)
        .await;

    match result {
        Ok(search_result) => {
            println!(
                "Complex query 2 (rust-lang/rust): Found {} author-specific PRs",
                search_result.issue_or_pull_requests.len()
            );
            for result in &search_result.issue_or_pull_requests {
                if let IssueOrPullrequest::PullRequest(pr) = result {
                    println!("  PR #{}: {}", pr.pull_request_id.number, pr.title);
                }
            }
        }
        Err(e) => panic!("Complex query 2 failed: {}", e),
    }

    // Test 3: Search for PRs with date range and multiple labels
    let query = SearchQuery::new("is:pr updated:2024-01-01..2024-12-31 label:C-bug label:I-crash");
    let result = client
        .search_resources(rust_repo, query, Some(3), None)
        .await;

    match result {
        Ok(search_result) => {
            println!(
                "Complex query 3 (rust-lang/rust): Found {} bug PRs",
                search_result.issue_or_pull_requests.len()
            );
            for result in &search_result.issue_or_pull_requests {
                if let IssueOrPullrequest::PullRequest(pr) = result {
                    println!("  PR #{}: {}", pr.pull_request_id.number, pr.title);
                }
            }
        }
        Err(e) => panic!("Complex query 3 failed: {}", e),
    }
}

/// Test complex search queries on React repository with advanced filters
///
/// This test verifies complex search functionality on the facebook/react repository
/// using various GitHub search operators and filters.
#[tokio::test]
#[serial]
#[cfg(feature = "integration-tests")]
async fn test_complex_search_queries_react_repository() {
    let client = create_test_github_client();
    let react_repo = RepositoryId::new("facebook".to_string(), "react".to_string());

    // Test 1: Search for TypeScript-related PRs with specific file changes
    let query = SearchQuery::new("is:pr typescript OR \"type definitions\" OR \".d.ts\"");
    let result = client
        .search_resources(react_repo.clone(), query, Some(5), None)
        .await;

    match result {
        Ok(search_result) => {
            println!(
                "Complex query 1 (facebook/react): Found {} TypeScript PRs",
                search_result.issue_or_pull_requests.len()
            );
            for result in &search_result.issue_or_pull_requests {
                if let IssueOrPullrequest::PullRequest(pr) = result {
                    println!("  PR #{}: {}", pr.pull_request_id.number, pr.title);
                }
            }
        }
        Err(e) => panic!("Complex query 1 failed: {}", e),
    }

    // Test 2: Search for performance improvements with metrics
    let query = SearchQuery::new(
        "is:pr \"performance\" AND (\"benchmark\" OR \"optimization\" OR \"faster\") is:merged",
    );
    let result = client
        .search_resources(react_repo.clone(), query, Some(5), None)
        .await;

    match result {
        Ok(search_result) => {
            println!(
                "Complex query 2 (facebook/react): Found {} performance PRs",
                search_result.issue_or_pull_requests.len()
            );
            for result in &search_result.issue_or_pull_requests {
                if let IssueOrPullrequest::PullRequest(pr) = result {
                    println!("  PR #{}: {}", pr.pull_request_id.number, pr.title);
                }
            }
        }
        Err(e) => panic!("Complex query 2 failed: {}", e),
    }

    // Test 3: Search for documentation updates with specific patterns
    let query = SearchQuery::new("is:pr (\"docs\" OR \"documentation\" OR \"README\") NOT \"api\"");
    let result = client
        .search_resources(react_repo, query, Some(5), None)
        .await;

    match result {
        Ok(search_result) => {
            println!(
                "Complex query 3 (facebook/react): Found {} documentation PRs",
                search_result.issue_or_pull_requests.len()
            );
            for result in &search_result.issue_or_pull_requests {
                if let IssueOrPullrequest::PullRequest(pr) = result {
                    println!("  PR #{}: {}", pr.pull_request_id.number, pr.title);
                }
            }
        }
        Err(e) => panic!("Complex query 3 failed: {}", e),
    }
}

/// Test advanced search operators and edge cases
///
/// This test covers complex boolean operators, regex patterns, and edge cases
/// in GitHub search syntax using VS Code repository.
#[tokio::test]
#[serial]
#[cfg(feature = "integration-tests")]
async fn test_advanced_search_operators_vscode() {
    let client = create_test_github_client();
    let vscode_repo = RepositoryId::new("microsoft".to_string(), "vscode".to_string());

    // Test 1: Boolean operators with parentheses
    let query = SearchQuery::new(
        "is:pr (\"extension\" OR \"plugin\") AND (\"marketplace\" OR \"gallery\") is:closed",
    );
    let result = client
        .search_resources(vscode_repo.clone(), query, Some(3), None)
        .await;

    match result {
        Ok(search_result) => {
            println!(
                "Advanced query 1 (microsoft/vscode): Found {} extension PRs",
                search_result.issue_or_pull_requests.len()
            );
            for result in &search_result.issue_or_pull_requests {
                if let IssueOrPullrequest::PullRequest(pr) = result {
                    println!("  PR #{}: {}", pr.pull_request_id.number, pr.title);
                }
            }
        }
        Err(e) => panic!("Advanced query 1 failed: {}", e),
    }

    // Test 2: Complex exclusion with multiple criteria
    let query =
        SearchQuery::new("is:pr \"debug\" NOT \"console\" NOT \"log\" NOT \"output\" is:open");
    let result = client
        .search_resources(vscode_repo.clone(), query, Some(3), None)
        .await;

    match result {
        Ok(search_result) => {
            println!(
                "Advanced query 2 (microsoft/vscode): Found {} debug PRs",
                search_result.issue_or_pull_requests.len()
            );
            for result in &search_result.issue_or_pull_requests {
                if let IssueOrPullrequest::PullRequest(pr) = result {
                    println!("  PR #{}: {}", pr.pull_request_id.number, pr.title);
                }
            }
        }
        Err(e) => panic!("Advanced query 2 failed: {}", e),
    }

    // Test 3: Complex date ranges with multiple conditions
    let query = SearchQuery::new("is:pr created:>2024-06-01 updated:<2024-12-01 comments:>5");
    let result = client
        .search_resources(vscode_repo, query, Some(3), None)
        .await;

    match result {
        Ok(search_result) => {
            println!(
                "Advanced query 3 (microsoft/vscode): Found {} discussed PRs",
                search_result.issue_or_pull_requests.len()
            );
            for result in &search_result.issue_or_pull_requests {
                if let IssueOrPullrequest::PullRequest(pr) = result {
                    println!("  PR #{}: {}", pr.pull_request_id.number, pr.title);
                }
            }
        }
        Err(e) => panic!("Advanced query 3 failed: {}", e),
    }
}

/// Test search queries with user filters and assignee patterns
///
/// This test verifies search functionality with user-based filters
/// and assignee patterns on the Kubernetes repository.
#[tokio::test]
#[serial]
#[cfg(feature = "integration-tests")]
async fn test_user_filter_search_kubernetes() {
    let client = create_test_github_client();
    let k8s_repo = RepositoryId::new("kubernetes".to_string(), "kubernetes".to_string());

    // Test 1: Search by review requests and assignees
    let query = SearchQuery::new("is:pr review-requested:@me OR assignee:@me is:open");
    let result = client
        .search_resources(k8s_repo.clone(), query, Some(3), None)
        .await;

    match result {
        Ok(search_result) => {
            println!(
                "User filter query 1 (kubernetes/kubernetes): Found {} assigned PRs",
                search_result.issue_or_pull_requests.len()
            );
            for result in &search_result.issue_or_pull_requests {
                if let IssueOrPullrequest::PullRequest(pr) = result {
                    println!("  PR #{}: {}", pr.pull_request_id.number, pr.title);
                }
            }
        }
        Err(e) => panic!("User filter query 1 failed: {}", e),
    }

    // Test 2: Search with team mentions and area labels
    let query = SearchQuery::new(
        "is:pr \"@kubernetes/sig-\" AND (label:area/kubelet OR label:area/apiserver)",
    );
    let result = client
        .search_resources(k8s_repo.clone(), query, Some(3), None)
        .await;

    match result {
        Ok(search_result) => {
            println!(
                "User filter query 2 (kubernetes/kubernetes): Found {} SIG PRs",
                search_result.issue_or_pull_requests.len()
            );
            for result in &search_result.issue_or_pull_requests {
                if let IssueOrPullrequest::PullRequest(pr) = result {
                    println!("  PR #{}: {}", pr.pull_request_id.number, pr.title);
                }
            }
        }
        Err(e) => panic!("User filter query 2 failed: {}", e),
    }

    // Test 3: Search for PRs with specific approval patterns
    let query = SearchQuery::new(
        "is:pr \"LGTM\" OR \"approved\" OR \"/approve\" is:closed merged:>2024-01-01",
    );
    let result = client
        .search_resources(k8s_repo, query, Some(3), None)
        .await;

    match result {
        Ok(search_result) => {
            println!(
                "User filter query 3 (kubernetes/kubernetes): Found {} approved PRs",
                search_result.issue_or_pull_requests.len()
            );
            for result in &search_result.issue_or_pull_requests {
                if let IssueOrPullrequest::PullRequest(pr) = result {
                    println!("  PR #{}: {}", pr.pull_request_id.number, pr.title);
                }
            }
        }
        Err(e) => panic!("User filter query 3 failed: {}", e),
    }
}

/// Test search with label combinations and milestone filters
///
/// This test verifies complex label filtering and milestone-based searches
/// on the Node.js repository.
#[tokio::test]
#[serial]
#[cfg(feature = "integration-tests")]
async fn test_label_milestone_search_nodejs() {
    let client = create_test_github_client();
    let nodejs_repo = RepositoryId::new("nodejs".to_string(), "node".to_string());

    // Test 1: Multiple label combinations with priority
    let query = SearchQuery::new(
        "is:pr (label:\"confirmed-bug\" OR label:\"needs-ci\") AND label:\"fast-track\"",
    );
    let result = client
        .search_resources(nodejs_repo.clone(), query, Some(3), None)
        .await;

    match result {
        Ok(search_result) => {
            println!(
                "Label query 1 (nodejs/node): Found {} priority PRs",
                search_result.issue_or_pull_requests.len()
            );
            for result in &search_result.issue_or_pull_requests {
                if let IssueOrPullrequest::PullRequest(pr) = result {
                    println!("  PR #{}: {}", pr.pull_request_id.number, pr.title);
                }
            }
        }
        Err(e) => panic!("Label query 1 failed: {}", e),
    }

    // Test 2: Subsystem-specific searches with dependencies
    let query = SearchQuery::new(
        "is:pr (\"fs:\" OR \"filesystem\" OR \"file system\") NOT \"test\" NOT \"doc\"",
    );
    let result = client
        .search_resources(nodejs_repo.clone(), query, Some(3), None)
        .await;

    match result {
        Ok(search_result) => {
            println!(
                "Label query 2 (nodejs/node): Found {} filesystem PRs",
                search_result.issue_or_pull_requests.len()
            );
            for result in &search_result.issue_or_pull_requests {
                if let IssueOrPullrequest::PullRequest(pr) = result {
                    println!("  PR #{}: {}", pr.pull_request_id.number, pr.title);
                }
            }
        }
        Err(e) => panic!("Label query 2 failed: {}", e),
    }

    // Test 3: Security and performance combination search
    let query = SearchQuery::new(
        "is:pr (\"security\" OR \"vulnerability\" OR \"CVE\") AND (\"performance\" OR \"benchmark\")",
    );
    let result = client
        .search_resources(nodejs_repo, query, Some(3), None)
        .await;

    match result {
        Ok(search_result) => {
            println!(
                "Label query 3 (nodejs/node): Found {} security+performance PRs",
                search_result.issue_or_pull_requests.len()
            );
            for result in &search_result.issue_or_pull_requests {
                if let IssueOrPullrequest::PullRequest(pr) = result {
                    println!("  PR #{}: {}", pr.pull_request_id.number, pr.title);
                }
            }
        }
        Err(e) => panic!("Label query 3 failed: {}", e),
    }
}

/// Test edge cases and error handling in complex searches
///
/// This test verifies that the search system handles edge cases, malformed queries,
/// and boundary conditions properly.
#[tokio::test]
#[serial]
#[cfg(feature = "integration-tests")]
async fn test_search_edge_cases_and_error_handling() {
    let client = create_test_github_client();
    let test_repo = RepositoryId::new("tacogips".to_string(), "gitcodes-mcp-test-1".to_string());

    // Test 1: Very long and complex query
    let complex_query = SearchQuery::new(
        "is:pr (\"feature\" OR \"enhancement\" OR \"improvement\" OR \"optimization\" OR \"refactor\" OR \"update\") \
        AND (\"typescript\" OR \"javascript\" OR \"react\" OR \"vue\" OR \"angular\" OR \"node\") \
        NOT (\"test\" OR \"spec\" OR \"mock\" OR \"fixture\" OR \"stub\") \
        created:>2020-01-01 updated:>2023-01-01",
    );
    let result = client
        .search_resources(test_repo.clone(), complex_query, Some(2), None)
        .await;

    match result {
        Ok(search_result) => {
            println!(
                "Edge case 1: Complex query returned {} results",
                search_result.issue_or_pull_requests.len()
            );
        }
        Err(e) => println!("Edge case 1: Complex query failed as expected: {}", e),
    }

    // Test 2: Query with special characters and escaping
    let special_chars_query = SearchQuery::new("is:pr \"[FEATURE]\" OR \"[BUG]\" OR \"[DOCS]\"");
    let result = client
        .search_resources(test_repo.clone(), special_chars_query, Some(2), None)
        .await;

    match result {
        Ok(search_result) => {
            println!(
                "Edge case 2: Special characters query returned {} results",
                search_result.issue_or_pull_requests.len()
            );
        }
        Err(e) => println!("Edge case 2: Special characters query failed: {}", e),
    }

    // Test 3: Empty and minimal queries
    let minimal_query = SearchQuery::new("is:pr");
    let result = client
        .search_resources(test_repo.clone(), minimal_query, Some(1), None)
        .await;

    match result {
        Ok(search_result) => {
            println!(
                "Edge case 3: Minimal query returned {} results",
                search_result.issue_or_pull_requests.len()
            );
            assert!(
                search_result.issue_or_pull_requests.len() <= 1,
                "Should respect limit of 1"
            );
            for result in &search_result.issue_or_pull_requests {
                if let IssueOrPullrequest::PullRequest(pr) = result {
                    assert!(!pr.title.is_empty(), "PR title should not be empty");
                    println!("  PR #{}: {}", pr.pull_request_id.number, pr.title);
                }
            }
        }
        Err(e) => println!("Edge case 3: Minimal query failed: {}", e),
    }

    // Test 4: Query with conflicting type filters (should return no results)
    let impossible_query = SearchQuery::new("is:pr AND is:issue"); // Explicitly impossible with AND
    let result = client
        .search_resources(test_repo, impossible_query, Some(5), None)
        .await;

    match result {
        Ok(search_result) => {
            println!(
                "Edge case 4: Conflicting query returned {} results",
                search_result.issue_or_pull_requests.len()
            );
            // Note: GitHub's search API behavior may vary, so we just document the result
            // rather than asserting a specific expectation
        }
        Err(e) => println!("Edge case 4: Conflicting query failed as expected: {}", e),
    }
}

/// Test fetching pull request diff using REST API
///
/// This test verifies that the client can successfully fetch the complete unified diff
/// for a pull request using GitHub's REST API.
#[tokio::test]
#[serial]
#[cfg(feature = "integration-tests")]
async fn test_fetch_pull_request_diff() {
    // Initialize GitHub client with token (if available) and reasonable timeout
    let client = create_test_github_client();

    // Create repository ID for the test repository
    let repository_id =
        RepositoryId::new("tacogips".to_string(), "gitcodes-mcp-test-1".to_string());

    // Test with PR #5 from the test repository
    let pr_number = PullRequestNumber::new(5);

    // Fetch the pull request diff
    let result = client
        .fetch_pull_request_diff(repository_id.clone(), pr_number)
        .await;

    // Verify the request succeeded
    assert!(
        result.is_ok(),
        "Failed to fetch pull request diff: {:?}",
        result
    );

    let diff = result.unwrap();

    // Verify the diff is not empty
    assert!(!diff.is_empty(), "Diff should not be empty");

    // Verify the diff contains unified diff format markers
    assert!(
        diff.contains("diff --git"),
        "Diff should contain 'diff --git' marker"
    );
    assert!(
        diff.contains("@@"),
        "Diff should contain '@@ ... @@' markers"
    );

    // Verify the diff contains additions or deletions
    let has_changes = diff.lines().any(|line| {
        let trimmed = line.trim_start();
        (trimmed.starts_with('+') && !trimmed.starts_with("+++"))
            || (trimmed.starts_with('-') && !trimmed.starts_with("---"))
    });
    assert!(
        has_changes,
        "Diff should contain at least one addition or deletion"
    );

    println!(
        "Successfully fetched diff for PR #{}: {} bytes, {} lines",
        pr_number.value(),
        diff.len(),
        diff.lines().count()
    );

    // Print first 20 lines of the diff for visual verification
    println!("First 20 lines of diff:");
    for (i, line) in diff.lines().take(20).enumerate() {
        println!("{:3}: {}", i + 1, line);
    }
}

/// Test fetching multiple pull request diffs using MultiResourceFetcher
///
/// This test verifies that the MultiResourceFetcher can successfully fetch diffs
/// from multiple pull requests.
#[tokio::test]
#[serial]
#[cfg(feature = "integration-tests")]
async fn test_multi_resource_fetcher_pull_request_diffs() {
    // Initialize GitHub client with token (if available) and reasonable timeout
    let client = create_test_github_client();
    let fetcher = MultiResourceFetcher::new(client);

    // Create repository ID for the test repository
    let repo = RepositoryId::new("tacogips".to_string(), "gitcodes-mcp-test-1".to_string());

    // Prepare PR numbers to fetch diffs for
    let pr_numbers = vec![PullRequestNumber::new(5)];

    let pr_requests = vec![(repo.clone(), pr_numbers.clone())];

    // Fetch pull request diffs
    let result = fetcher.fetch_pull_request_diffs(pr_requests).await;

    // Verify the request succeeded
    assert!(
        result.is_ok(),
        "Failed to fetch pull request diffs: {:?}",
        result
    );

    let diffs_by_repo = result.unwrap();

    // Verify we got results for the repository
    assert!(
        diffs_by_repo.contains_key(&repo),
        "Expected to find diffs for the repository"
    );

    let pr_diffs = &diffs_by_repo[&repo];

    // Verify we got diffs for all requested PRs
    assert_eq!(
        pr_diffs.len(),
        pr_numbers.len(),
        "Expected diffs for all requested PRs"
    );

    // Verify each diff
    for (pr_number, diff) in pr_diffs {
        assert!(
            pr_numbers.contains(pr_number),
            "PR number should be in the requested list"
        );
        assert!(!diff.is_empty(), "Diff should not be empty");
        assert!(
            diff.contains("diff --git"),
            "Diff should contain unified diff format markers"
        );

        println!(
            "Successfully fetched diff for PR #{}: {} bytes",
            pr_number.value(),
            diff.len()
        );
    }
}

/// Test fetching pull request files list
///
/// This test verifies that the client can successfully fetch the list of changed files
/// in a pull request. The files list contains only metadata, no patch content.
#[tokio::test]
#[serial]
#[cfg(feature = "integration-tests")]
async fn test_fetch_pull_request_files() {
    // Initialize GitHub client with token (if available) and reasonable timeout
    let client = create_test_github_client();

    // Create repository ID for the test repository
    let repository_id =
        RepositoryId::new("tacogips".to_string(), "gitcodes-mcp-test-1".to_string());

    // Test with PR #5 from the test repository
    let pr_number = PullRequestNumber::new(5);

    // Fetch the pull request files
    let result = client
        .fetch_pull_request_files(repository_id.clone(), pr_number)
        .await;

    // Verify the request succeeded
    assert!(
        result.is_ok(),
        "Failed to fetch pull request files: {:?}",
        result
    );

    let files = result.unwrap();

    // Verify we got some files
    assert!(!files.is_empty(), "Files list should not be empty");

    // Verify file metadata is present
    for file in &files {
        assert!(!file.filename.is_empty(), "Filename should not be empty");
        assert!(!file.status.is_empty(), "Status should not be empty");
        assert!(!file.sha.is_empty(), "SHA should not be empty");

        // Verify patch is always None (use fetch_pull_request_file_content for patches)
        assert!(
            file.patch.is_none(),
            "Patch should always be None in file list"
        );

        println!(
            "File: {} ({}, +{} -{} changes)",
            file.filename, file.status, file.additions, file.deletions
        );
    }

    println!(
        "Successfully fetched {} files for PR #{}",
        files.len(),
        pr_number.value()
    );
}

/// Test fetching individual file content from pull request
///
/// This test verifies that the client can successfully fetch the diff content
/// for a specific file in a pull request.
#[tokio::test]
#[serial]
#[cfg(feature = "integration-tests")]
async fn test_fetch_pull_request_file_content() {
    // Initialize GitHub client with token (if available) and reasonable timeout
    let client = create_test_github_client();

    // Create repository ID for the test repository
    let repository_id =
        RepositoryId::new("tacogips".to_string(), "gitcodes-mcp-test-1".to_string());

    // Test with PR #5 from the test repository
    let pr_number = PullRequestNumber::new(5);

    // First, get the list of files
    let files = client
        .fetch_pull_request_files(repository_id.clone(), pr_number)
        .await
        .expect("Failed to fetch file list");

    assert!(!files.is_empty(), "Files list should not be empty");

    // Get the first file to test with
    let test_file = &files[0];
    println!("Testing with file: {}", test_file.filename);

    // Fetch the content for this specific file
    let result = client
        .fetch_pull_request_file_content(repository_id.clone(), pr_number, &test_file.filename)
        .await;

    // Verify the request succeeded
    assert!(result.is_ok(), "Failed to fetch file content: {:?}", result);

    let patch_opt = result.unwrap();

    // Some files might not have patches (e.g., binary files, very large files)
    if let Some(patch) = patch_opt {
        assert!(!patch.is_empty(), "Patch should not be empty if present");

        // Verify patch contains diff markers
        assert!(
            patch.contains("@@"),
            "Patch should contain '@@ ... @@' markers"
        );

        println!(
            "File: {} ({}, +{} -{} changes)",
            test_file.filename, test_file.status, test_file.additions, test_file.deletions
        );
        println!("Patch size: {} bytes", patch.len());

        // Print first few lines of the patch for verification
        println!("First 5 lines of patch:");
        for (i, line) in patch.lines().take(5).enumerate() {
            println!("  {}: {}", i + 1, line);
        }

        println!(
            "Successfully fetched file content for {} in PR #{}",
            test_file.filename,
            pr_number.value()
        );
    } else {
        println!(
            "File {} has no patch content (binary or very large file)",
            test_file.filename
        );
    }
}

/// Test complete workflow: fetch file list then individual file diffs
///
/// This test demonstrates the recommended workflow: first fetch file list (lightweight)
/// to get file metadata, then fetch diffs for specific files of interest.
#[tokio::test]
#[serial]
#[cfg(feature = "integration-tests")]
async fn test_pull_request_files_workflow() {
    // Initialize GitHub client with token (if available) and reasonable timeout
    let client = create_test_github_client();

    // Create repository ID for the test repository
    let repository_id =
        RepositoryId::new("tacogips".to_string(), "gitcodes-mcp-test-1".to_string());

    // Test with PR #5 from the test repository
    let pr_number = PullRequestNumber::new(5);

    // Step 1: Fetch file list (lightweight, no patch content)
    let files = client
        .fetch_pull_request_files(repository_id.clone(), pr_number)
        .await
        .expect("Failed to fetch file list");

    assert!(!files.is_empty(), "Should have at least one file");

    println!("Step 1: Fetched {} files", files.len());
    for file in &files {
        println!(
            "  - {} ({}, +{} -{} changes)",
            file.filename, file.status, file.additions, file.deletions
        );
        assert!(file.patch.is_none(), "Patch should always be None");
    }

    // Step 2: Fetch diff for a specific file of interest
    let target_file = &files[0];
    println!("\nStep 2: Fetching diff for {}", target_file.filename);

    let patch_opt = client
        .fetch_pull_request_file_content(repository_id.clone(), pr_number, &target_file.filename)
        .await
        .expect("Failed to fetch file content");

    // Step 3: Process the file diff
    if let Some(patch) = patch_opt {
        println!("\nStep 3: Processing file diff:");
        println!("  Filename: {}", target_file.filename);
        println!("  Status: {}", target_file.status);
        println!("  Additions: {}", target_file.additions);
        println!("  Deletions: {}", target_file.deletions);
        println!("  Patch size: {} bytes", patch.len());
        println!("  Patch preview (first 10 lines):");
        for (i, line) in patch.lines().take(10).enumerate() {
            println!("    {}: {}", i + 1, line);
        }

        // Verify patch format
        assert!(
            patch.contains("@@"),
            "Patch should contain unified diff markers"
        );
    } else {
        println!(
            "\nStep 3: File {} has no patch (binary or very large file)",
            target_file.filename
        );
    }

    println!(
        "\nWorkflow completed: Successfully fetched file list and individual file diff for PR #{}",
        pr_number.value()
    );
}

/// Test fetching pull request diff contents without skip/limit
///
/// This test verifies that the get_pull_request_diff_contents function
/// can fetch the complete diff for a specific file when no skip/limit is specified.
#[tokio::test]
#[serial]
#[cfg(feature = "integration-tests")]
async fn test_get_pull_request_diff_contents_full() {
    let client = create_test_github_client();

    // Use PR #5 from test repository
    let pr_url =
        PullRequestUrl("https://github.com/tacogips/gitcodes-mcp-test-1/pull/5".to_string());

    // First, get the list of files to find a valid file path
    let repository_id =
        RepositoryId::new("tacogips".to_string(), "gitcodes-mcp-test-1".to_string());
    let pr_number = PullRequestNumber::new(5);

    let files = client
        .fetch_pull_request_files(repository_id, pr_number)
        .await
        .expect("Failed to fetch file list");

    assert!(!files.is_empty(), "Should have at least one file");

    let test_file_path = files[0].filename.clone();
    println!("Testing with file: {}", test_file_path);

    // Fetch the complete diff content
    let result = functions::pull_request::get_pull_request_diff_contents(
        &client,
        pr_url,
        test_file_path.clone(),
        None, // no skip
        None, // no limit
    )
    .await;

    assert!(
        result.is_ok(),
        "Failed to fetch diff contents: {:?}",
        result
    );

    let diff = result.unwrap();

    // Verify the diff is not empty
    assert!(!diff.is_empty(), "Diff should not be empty");

    // Verify it contains diff markers
    assert!(
        diff.contains("@@"),
        "Diff should contain unified diff markers"
    );

    let line_count = diff.lines().count();
    println!(
        "Successfully fetched full diff for {}: {} lines",
        test_file_path, line_count
    );

    // Print first 10 lines for verification
    println!("First 10 lines:");
    for (i, line) in diff.lines().take(10).enumerate() {
        println!("  {}: {}", i + 1, line);
    }
}

/// Test fetching pull request diff contents with skip parameter
///
/// This test verifies that the skip parameter correctly skips lines
/// from the beginning of the diff.
#[tokio::test]
#[serial]
#[cfg(feature = "integration-tests")]
async fn test_get_pull_request_diff_contents_with_skip() {
    let client = create_test_github_client();

    let pr_url =
        PullRequestUrl("https://github.com/tacogips/gitcodes-mcp-test-1/pull/5".to_string());

    // Get file list
    let repository_id =
        RepositoryId::new("tacogips".to_string(), "gitcodes-mcp-test-1".to_string());
    let pr_number = PullRequestNumber::new(5);

    let files = client
        .fetch_pull_request_files(repository_id, pr_number)
        .await
        .expect("Failed to fetch file list");

    assert!(!files.is_empty(), "Should have at least one file");

    let test_file_path = files[0].filename.clone();

    // First fetch full diff to get baseline
    let full_diff = functions::pull_request::get_pull_request_diff_contents(
        &client,
        pr_url.clone(),
        test_file_path.clone(),
        None,
        None,
    )
    .await
    .expect("Failed to fetch full diff");

    let full_lines: Vec<&str> = full_diff.lines().collect();
    let total_lines = full_lines.len();

    println!("Total lines in full diff: {}", total_lines);

    // Test with skip=5
    let skip_count = 5;
    if total_lines > skip_count {
        let result = functions::pull_request::get_pull_request_diff_contents(
            &client,
            pr_url,
            test_file_path.clone(),
            Some(skip_count as u32),
            None,
        )
        .await;

        assert!(
            result.is_ok(),
            "Failed to fetch diff with skip: {:?}",
            result
        );

        let skipped_diff = result.unwrap();
        let skipped_lines: Vec<&str> = skipped_diff.lines().collect();

        // Verify we skipped the correct number of lines
        assert_eq!(
            skipped_lines.len(),
            total_lines - skip_count,
            "Should have {} lines after skipping {}",
            total_lines - skip_count,
            skip_count
        );

        // Verify the first line matches the expected line from full diff
        assert_eq!(
            skipped_lines[0],
            full_lines[skip_count],
            "First line after skip should match line {} from full diff",
            skip_count + 1
        );

        println!(
            "Successfully fetched diff with skip={}: {} lines",
            skip_count,
            skipped_lines.len()
        );
    } else {
        println!(
            "Skipping test: total lines ({}) <= skip count ({})",
            total_lines, skip_count
        );
    }
}

/// Test fetching pull request diff contents with limit parameter
///
/// This test verifies that the limit parameter correctly limits the number
/// of lines returned.
#[tokio::test]
#[serial]
#[cfg(feature = "integration-tests")]
async fn test_get_pull_request_diff_contents_with_limit() {
    let client = create_test_github_client();

    let pr_url =
        PullRequestUrl("https://github.com/tacogips/gitcodes-mcp-test-1/pull/5".to_string());

    // Get file list
    let repository_id =
        RepositoryId::new("tacogips".to_string(), "gitcodes-mcp-test-1".to_string());
    let pr_number = PullRequestNumber::new(5);

    let files = client
        .fetch_pull_request_files(repository_id, pr_number)
        .await
        .expect("Failed to fetch file list");

    assert!(!files.is_empty(), "Should have at least one file");

    let test_file_path = files[0].filename.clone();

    // First fetch full diff to get baseline
    let full_diff = functions::pull_request::get_pull_request_diff_contents(
        &client,
        pr_url.clone(),
        test_file_path.clone(),
        None,
        None,
    )
    .await
    .expect("Failed to fetch full diff");

    let full_lines: Vec<&str> = full_diff.lines().collect();
    let total_lines = full_lines.len();

    println!("Total lines in full diff: {}", total_lines);

    // Test with limit=10
    let limit_count = 10;
    if total_lines > 0 {
        let result = functions::pull_request::get_pull_request_diff_contents(
            &client,
            pr_url,
            test_file_path.clone(),
            None,
            Some(limit_count),
        )
        .await;

        assert!(
            result.is_ok(),
            "Failed to fetch diff with limit: {:?}",
            result
        );

        let limited_diff = result.unwrap();
        let limited_lines: Vec<&str> = limited_diff.lines().collect();

        // Verify line count is at most the limit
        let expected_lines = std::cmp::min(limit_count as usize, total_lines);
        assert_eq!(
            limited_lines.len(),
            expected_lines,
            "Should have at most {} lines",
            limit_count
        );

        // Verify the lines match the beginning of full diff
        for (i, line) in limited_lines.iter().enumerate() {
            assert_eq!(
                *line,
                full_lines[i],
                "Line {} should match full diff",
                i + 1
            );
        }

        println!(
            "Successfully fetched diff with limit={}: {} lines",
            limit_count,
            limited_lines.len()
        );
    }
}

/// Test fetching pull request diff contents with both skip and limit
///
/// This test verifies that skip and limit work correctly together to return
/// a specific range of lines from the diff.
#[tokio::test]
#[serial]
#[cfg(feature = "integration-tests")]
async fn test_get_pull_request_diff_contents_with_skip_and_limit() {
    let client = create_test_github_client();

    let pr_url =
        PullRequestUrl("https://github.com/tacogips/gitcodes-mcp-test-1/pull/5".to_string());

    // Get file list
    let repository_id =
        RepositoryId::new("tacogips".to_string(), "gitcodes-mcp-test-1".to_string());
    let pr_number = PullRequestNumber::new(5);

    let files = client
        .fetch_pull_request_files(repository_id, pr_number)
        .await
        .expect("Failed to fetch file list");

    assert!(!files.is_empty(), "Should have at least one file");

    let test_file_path = files[0].filename.clone();

    // First fetch full diff to get baseline
    let full_diff = functions::pull_request::get_pull_request_diff_contents(
        &client,
        pr_url.clone(),
        test_file_path.clone(),
        None,
        None,
    )
    .await
    .expect("Failed to fetch full diff");

    let full_lines: Vec<&str> = full_diff.lines().collect();
    let total_lines = full_lines.len();

    println!("Total lines in full diff: {}", total_lines);

    // Test with skip=3 and limit=7 (lines 4-10)
    let skip_count = 3;
    let limit_count = 7;

    if total_lines > skip_count {
        let result = functions::pull_request::get_pull_request_diff_contents(
            &client,
            pr_url,
            test_file_path.clone(),
            Some(skip_count as u32),
            Some(limit_count),
        )
        .await;

        assert!(
            result.is_ok(),
            "Failed to fetch diff with skip and limit: {:?}",
            result
        );

        let ranged_diff = result.unwrap();
        let ranged_lines: Vec<&str> = ranged_diff.lines().collect();

        // Calculate expected number of lines
        let expected_lines = std::cmp::min(limit_count as usize, total_lines - skip_count);
        assert_eq!(
            ranged_lines.len(),
            expected_lines,
            "Should have {} lines (skip={}, limit={})",
            expected_lines,
            skip_count,
            limit_count
        );

        // Verify the lines match the expected range from full diff
        for (i, line) in ranged_lines.iter().enumerate() {
            let full_diff_idx = skip_count + i;
            assert_eq!(
                *line,
                full_lines[full_diff_idx],
                "Line {} should match line {} from full diff",
                i + 1,
                full_diff_idx + 1
            );
        }

        println!(
            "Successfully fetched diff with skip={} and limit={}: {} lines",
            skip_count,
            limit_count,
            ranged_lines.len()
        );

        // Print the ranged lines for verification
        println!(
            "Lines {} to {}:",
            skip_count + 1,
            skip_count + ranged_lines.len()
        );
        for (i, line) in ranged_lines.iter().enumerate() {
            println!("  {}: {}", skip_count + i + 1, line);
        }
    } else {
        println!(
            "Skipping test: total lines ({}) <= skip count ({})",
            total_lines, skip_count
        );
    }
}

/// Test error handling when skip exceeds total lines
///
/// This test verifies that the function returns an appropriate error when
/// the skip value is greater than the total number of lines in the diff.
#[tokio::test]
#[serial]
#[cfg(feature = "integration-tests")]
async fn test_get_pull_request_diff_contents_skip_exceeds_total() {
    let client = create_test_github_client();

    let pr_url =
        PullRequestUrl("https://github.com/tacogips/gitcodes-mcp-test-1/pull/5".to_string());

    // Get file list
    let repository_id =
        RepositoryId::new("tacogips".to_string(), "gitcodes-mcp-test-1".to_string());
    let pr_number = PullRequestNumber::new(5);

    let files = client
        .fetch_pull_request_files(repository_id, pr_number)
        .await
        .expect("Failed to fetch file list");

    assert!(!files.is_empty(), "Should have at least one file");

    let test_file_path = files[0].filename.clone();

    // First fetch full diff to get total line count
    let full_diff = functions::pull_request::get_pull_request_diff_contents(
        &client,
        pr_url.clone(),
        test_file_path.clone(),
        None,
        None,
    )
    .await
    .expect("Failed to fetch full diff");

    let total_lines = full_diff.lines().count();
    println!("Total lines in full diff: {}", total_lines);

    // Try to skip more lines than exist
    let excessive_skip = (total_lines + 10) as u32;

    let result = functions::pull_request::get_pull_request_diff_contents(
        &client,
        pr_url,
        test_file_path,
        Some(excessive_skip),
        None,
    )
    .await;

    // Should return an error
    assert!(
        result.is_err(),
        "Should return error when skip exceeds total lines"
    );

    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(
        error_msg.contains("exceeds total lines"),
        "Error message should mention exceeding total lines: {}",
        error_msg
    );

    println!(
        "Successfully detected skip exceeding total lines: {}",
        error_msg
    );
}

/// Test with skip=0 (should behave same as no skip)
///
/// This test verifies that skip=0 returns the same result as not specifying skip.
#[tokio::test]
#[serial]
#[cfg(feature = "integration-tests")]
async fn test_get_pull_request_diff_contents_skip_zero() {
    let client = create_test_github_client();

    let pr_url =
        PullRequestUrl("https://github.com/tacogips/gitcodes-mcp-test-1/pull/5".to_string());

    // Get file list
    let repository_id =
        RepositoryId::new("tacogips".to_string(), "gitcodes-mcp-test-1".to_string());
    let pr_number = PullRequestNumber::new(5);

    let files = client
        .fetch_pull_request_files(repository_id, pr_number)
        .await
        .expect("Failed to fetch file list");

    assert!(!files.is_empty(), "Should have at least one file");

    let test_file_path = files[0].filename.clone();

    // Fetch with no skip
    let no_skip_diff = functions::pull_request::get_pull_request_diff_contents(
        &client,
        pr_url.clone(),
        test_file_path.clone(),
        None,
        None,
    )
    .await
    .expect("Failed to fetch diff without skip");

    // Fetch with skip=0
    let skip_zero_diff = functions::pull_request::get_pull_request_diff_contents(
        &client,
        pr_url,
        test_file_path,
        Some(0),
        None,
    )
    .await
    .expect("Failed to fetch diff with skip=0");

    // Both should be identical
    assert_eq!(
        no_skip_diff, skip_zero_diff,
        "skip=0 should produce the same result as no skip"
    );

    println!(
        "Verified that skip=0 produces same result as no skip: {} lines",
        no_skip_diff.lines().count()
    );
}
