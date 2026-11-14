//! Core type system and domain definitions
//!
//! This module provides the central type definitions for the GitHub Insight system,
//! following domain-driven design principles. All types are strongly-typed and
//! provide comprehensive validation and conversion capabilities.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::EnumString;

pub use crate::github::graphql::graphql_types::repository::MilestoneNumber;

pub mod issue;
pub mod label;
pub mod profile;
pub mod project;
pub mod pull_request;
pub mod repository;
pub mod search;
pub mod user;

pub use issue::*;
pub use profile::*;
pub use project::*;
pub use pull_request::*;
pub use repository::*;
pub use search::*;
pub use user::*;

use once_cell::sync::Lazy;
use regex::Regex;

static ISSUE_PR_URL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:https?://)?github\.com/([^/\s]+)/([^/\s]+)/(?:pull|issues)/(\d+)")
        .expect("Failed to compile GitHub URL regex")
});

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum IssueOrPullrequestId {
    IssueId(IssueId),
    PullrequestId(PullRequestId),
}

impl IssueOrPullrequestId {
    pub fn extract_resource_url_from_text(text: &str) -> Vec<IssueOrPullrequestId> {
        let mut results = Vec::new();

        for captures in ISSUE_PR_URL_REGEX.captures_iter(text) {
            let number = captures.get(3).unwrap().as_str();

            if number.parse::<u32>().is_ok() {
                let full_match = captures.get(0).unwrap().as_str();
                if full_match.contains("/pull/") {
                    // Parse as pull request
                    let pr_url = PullRequestUrl(full_match.to_string());
                    if let Ok(pr_id) = PullRequestId::parse_url(&pr_url) {
                        results.push(IssueOrPullrequestId::PullrequestId(pr_id));
                    }
                } else if full_match.contains("/issues/") {
                    // Parse as issue
                    let issue_url = IssueUrl(full_match.to_string());
                    if let Ok(issue_id) = IssueId::parse_url(&issue_url) {
                        results.push(IssueOrPullrequestId::IssueId(issue_id));
                    }
                }
            }
        }

        results
    }
    pub fn url(&self) -> String {
        match self {
            IssueOrPullrequestId::IssueId(issue_id) => issue_id.url(),
            IssueOrPullrequestId::PullrequestId(pr_id) => pr_id.url(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueOrPullrequest {
    Issue(Issue),
    PullRequest(PullRequest),
}

pub struct SearchResult {
    pub repository_id: RepositoryId,
    pub issue_or_pull_requests: Vec<crate::types::IssueOrPullrequest>,
    pub next_pager: Option<SearchResultPager>,
}

/// Output format options for search results
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Default, EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum OutputOption {
    /// Light format with minimal information
    #[default]
    Light,
    /// Rich format with comprehensive details
    Rich,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_resource_url_from_text_single_issue() {
        let text = "Related issue: https://github.com/rust-lang/rust/issues/12345";
        let results = IssueOrPullrequestId::extract_resource_url_from_text(text);

        assert_eq!(results.len(), 1);
        match &results[0] {
            IssueOrPullrequestId::IssueId(issue_id) => {
                assert_eq!(issue_id.git_repository.owner, "rust-lang".into());
                assert_eq!(issue_id.git_repository.repository_name, "rust".into());
                assert_eq!(issue_id.number, 12345);
            }
            _ => panic!("Expected IssueId"),
        }
    }

    #[test]
    fn test_extract_resource_url_from_text_single_pull_request() {
        let text = "関連PR https://github.com/microsoft/vscode/pull/3604";
        let results = IssueOrPullrequestId::extract_resource_url_from_text(text);

        assert_eq!(results.len(), 1);
        match &results[0] {
            IssueOrPullrequestId::PullrequestId(pr_id) => {
                assert_eq!(pr_id.git_repository.owner, "microsoft".into());
                assert_eq!(pr_id.git_repository.repository_name, "vscode".into());
                assert_eq!(pr_id.number, 3604);
            }
            _ => panic!("Expected PullrequestId"),
        }
    }

    #[test]
    fn test_extract_resource_url_from_text_multiple_resources() {
        let text = r#"
        This is related to issue https://github.com/rust-lang/rust/issues/12345
        and also PR https://github.com/microsoft/vscode/pull/3604.
        Another issue: https://github.com/facebook/react/issues/9876
        "#;
        let results = IssueOrPullrequestId::extract_resource_url_from_text(text);

        assert_eq!(results.len(), 3);

        // Check first result (issue)
        match &results[0] {
            IssueOrPullrequestId::IssueId(issue_id) => {
                assert_eq!(issue_id.git_repository.owner, "rust-lang".into());
                assert_eq!(issue_id.git_repository.repository_name, "rust".into());
                assert_eq!(issue_id.number, 12345);
            }
            _ => panic!("Expected IssueId"),
        }

        // Check second result (PR)
        match &results[1] {
            IssueOrPullrequestId::PullrequestId(pr_id) => {
                assert_eq!(pr_id.git_repository.owner, "microsoft".into());
                assert_eq!(pr_id.git_repository.repository_name, "vscode".into());
                assert_eq!(pr_id.number, 3604);
            }
            _ => panic!("Expected PullrequestId"),
        }

        // Check third result (issue)
        match &results[2] {
            IssueOrPullrequestId::IssueId(issue_id) => {
                assert_eq!(issue_id.git_repository.owner, "facebook".into());
                assert_eq!(issue_id.git_repository.repository_name, "react".into());
                assert_eq!(issue_id.number, 9876);
            }
            _ => panic!("Expected IssueId"),
        }
    }

    #[test]
    fn test_extract_resource_url_from_text_without_protocol() {
        let text =
            "See github.com/rust-lang/rust/issues/12345 and github.com/microsoft/vscode/pull/3604";
        let results = IssueOrPullrequestId::extract_resource_url_from_text(text);

        assert_eq!(results.len(), 2);

        match &results[0] {
            IssueOrPullrequestId::IssueId(issue_id) => {
                assert_eq!(issue_id.git_repository.owner, "rust-lang".into());
                assert_eq!(issue_id.git_repository.repository_name, "rust".into());
                assert_eq!(issue_id.number, 12345);
            }
            _ => panic!("Expected IssueId"),
        }

        match &results[1] {
            IssueOrPullrequestId::PullrequestId(pr_id) => {
                assert_eq!(pr_id.git_repository.owner, "microsoft".into());
                assert_eq!(pr_id.git_repository.repository_name, "vscode".into());
                assert_eq!(pr_id.number, 3604);
            }
            _ => panic!("Expected PullrequestId"),
        }
    }

    #[test]
    fn test_extract_resource_url_from_text_empty_text() {
        let text = "";
        let results = IssueOrPullrequestId::extract_resource_url_from_text(text);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_extract_resource_url_from_text_no_matches() {
        let text = "This text has no GitHub URLs in it at all.";
        let results = IssueOrPullrequestId::extract_resource_url_from_text(text);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_extract_resource_url_from_text_invalid_number() {
        let text = "https://github.com/rust-lang/rust/issues/invalid";
        let results = IssueOrPullrequestId::extract_resource_url_from_text(text);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_extract_resource_url_from_text_mixed_protocols() {
        let text = r#"
        HTTP issue: http://github.com/rust-lang/rust/issues/12345
        HTTPS PR: https://github.com/microsoft/vscode/pull/3604
        No protocol: github.com/facebook/react/issues/9876
        "#;
        let results = IssueOrPullrequestId::extract_resource_url_from_text(text);

        assert_eq!(results.len(), 3);

        // Verify all three were parsed correctly
        let issue_count = results
            .iter()
            .filter(|r| matches!(r, IssueOrPullrequestId::IssueId(_)))
            .count();
        let pr_count = results
            .iter()
            .filter(|r| matches!(r, IssueOrPullrequestId::PullrequestId(_)))
            .count();

        assert_eq!(issue_count, 2);
        assert_eq!(pr_count, 1);
    }
}
