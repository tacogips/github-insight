//! Tests for RepositoryBranchPair and related types
//!
//! These tests verify the parsing, validation, and functionality of repository branch
//! pairs, group names, and related type conversions.

use github_insight::types::{
    Branch,
    profile::{GroupName, RepositoryBranchPair},
    repository::{Owner, RepositoryId, RepositoryName},
};

#[test]
fn test_repository_branch_pair_creation() {
    let repository_id = RepositoryId {
        owner: Owner::from("rust-lang"),
        repository_name: RepositoryName::from("rust"),
    };
    let branch = Branch::new("main");

    let pair = RepositoryBranchPair::new(repository_id.clone(), branch.clone());

    assert_eq!(pair.repository_id, repository_id);
    assert_eq!(pair.branch, branch);
}

#[test]
fn test_repository_branch_pair_try_from_str_valid_format() {
    let specifier = "https://github.com/rust-lang/rust@main";
    let pair = RepositoryBranchPair::try_from_str(specifier).unwrap();

    assert_eq!(pair.repository_id.owner.as_str(), "rust-lang");
    assert_eq!(pair.repository_id.repository_name.as_str(), "rust");
    assert_eq!(pair.branch.as_str(), "main");
}

#[test]
fn test_repository_branch_pair_try_from_str_with_different_branch() {
    let specifier = "https://github.com/tokio-rs/tokio@master";
    let pair = RepositoryBranchPair::try_from_str(specifier).unwrap();

    assert_eq!(pair.repository_id.owner.as_str(), "tokio-rs");
    assert_eq!(pair.repository_id.repository_name.as_str(), "tokio");
    assert_eq!(pair.branch.as_str(), "master");
}

#[test]
fn test_repository_branch_pair_try_from_str_with_feature_branch() {
    let specifier = "https://github.com/serde-rs/serde@feature/async-support";
    let pair = RepositoryBranchPair::try_from_str(specifier).unwrap();

    assert_eq!(pair.repository_id.owner.as_str(), "serde-rs");
    assert_eq!(pair.repository_id.repository_name.as_str(), "serde");
    assert_eq!(pair.branch.as_str(), "feature/async-support");
}

#[test]
fn test_repository_branch_pair_try_from_str_invalid_format_no_at() {
    let specifier = "https://github.com/rust-lang/rust";
    let result = RepositoryBranchPair::try_from_str(specifier);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Invalid repository branch specifier format"));
    assert!(error_msg.contains("Expected format: 'repo_url@branch'"));
}

#[test]
fn test_repository_branch_pair_try_from_str_invalid_format_multiple_at() {
    let specifier = "https://github.com/rust-lang/rust@main@dev";
    let result = RepositoryBranchPair::try_from_str(specifier);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Invalid repository branch specifier format"));
}

#[test]
fn test_repository_branch_pair_try_from_str_empty_repo_url() {
    let specifier = "@main";
    let result = RepositoryBranchPair::try_from_str(specifier);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Repository URL cannot be empty"));
}

#[test]
fn test_repository_branch_pair_try_from_str_empty_branch() {
    let specifier = "https://github.com/rust-lang/rust@";
    let result = RepositoryBranchPair::try_from_str(specifier);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Branch name cannot be empty"));
}

#[test]
fn test_repository_branch_pair_try_from_str_whitespace_handling() {
    let specifier = "  https://github.com/rust-lang/rust@main  ";
    let pair = RepositoryBranchPair::try_from_str(specifier).unwrap();

    assert_eq!(pair.repository_id.owner.as_str(), "rust-lang");
    assert_eq!(pair.repository_id.repository_name.as_str(), "rust");
    assert_eq!(pair.branch.as_str(), "main");
}

#[test]
fn test_repository_branch_pair_try_from_str_whitespace_around_at() {
    let specifier = "https://github.com/rust-lang/rust @ main";
    let pair = RepositoryBranchPair::try_from_str(specifier).unwrap();

    assert_eq!(pair.repository_id.owner.as_str(), "rust-lang");
    assert_eq!(pair.repository_id.repository_name.as_str(), "rust");
    assert_eq!(pair.branch.as_str(), "main");
}

#[test]
fn test_repository_branch_pair_try_from_specifiers_single() {
    let specifiers = vec!["https://github.com/rust-lang/rust@main".to_string()];
    let pairs = RepositoryBranchPair::try_from_specifiers(&specifiers).unwrap();

    assert_eq!(pairs.len(), 1);
    assert_eq!(pairs[0].repository_id.owner.as_str(), "rust-lang");
    assert_eq!(pairs[0].repository_id.repository_name.as_str(), "rust");
    assert_eq!(pairs[0].branch.as_str(), "main");
}

#[test]
fn test_repository_branch_pair_try_from_specifiers_multiple() {
    let specifiers = vec![
        "https://github.com/rust-lang/rust@main".to_string(),
        "https://github.com/tokio-rs/tokio@master".to_string(),
        "https://github.com/serde-rs/serde@main".to_string(),
    ];
    let pairs = RepositoryBranchPair::try_from_specifiers(&specifiers).unwrap();

    assert_eq!(pairs.len(), 3);

    assert_eq!(pairs[0].repository_id.owner.as_str(), "rust-lang");
    assert_eq!(pairs[0].repository_id.repository_name.as_str(), "rust");
    assert_eq!(pairs[0].branch.as_str(), "main");

    assert_eq!(pairs[1].repository_id.owner.as_str(), "tokio-rs");
    assert_eq!(pairs[1].repository_id.repository_name.as_str(), "tokio");
    assert_eq!(pairs[1].branch.as_str(), "master");

    assert_eq!(pairs[2].repository_id.owner.as_str(), "serde-rs");
    assert_eq!(pairs[2].repository_id.repository_name.as_str(), "serde");
    assert_eq!(pairs[2].branch.as_str(), "main");
}

#[test]
fn test_repository_branch_pair_try_from_specifiers_empty() {
    let specifiers: Vec<String> = vec![];
    let pairs = RepositoryBranchPair::try_from_specifiers(&specifiers).unwrap();

    assert_eq!(pairs.len(), 0);
}

#[test]
fn test_repository_branch_pair_try_from_specifiers_with_invalid() {
    let specifiers = vec![
        "https://github.com/rust-lang/rust@main".to_string(),
        "invalid-format".to_string(),
    ];
    let result = RepositoryBranchPair::try_from_specifiers(&specifiers);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Invalid repository branch specifier format"));
}

#[test]
fn test_repository_branch_pair_display() {
    let pair =
        RepositoryBranchPair::try_from_str("https://github.com/rust-lang/rust@main").unwrap();
    let display_str = pair.to_string();

    assert_eq!(display_str, "https://github.com/rust-lang/rust@main");
}

#[test]
fn test_repository_branch_pair_display_different_formats() {
    let test_cases = vec![
        "https://github.com/rust-lang/rust@main",
        "https://github.com/tokio-rs/tokio@master",
        "https://github.com/serde-rs/serde@feature/async-support",
    ];

    for specifier in test_cases {
        let pair = RepositoryBranchPair::try_from_str(specifier).unwrap();
        assert_eq!(pair.to_string(), specifier);
    }
}

#[test]
fn test_repository_branch_pair_equality() {
    let pair1 =
        RepositoryBranchPair::try_from_str("https://github.com/rust-lang/rust@main").unwrap();
    let pair2 =
        RepositoryBranchPair::try_from_str("https://github.com/rust-lang/rust@main").unwrap();
    let pair3 =
        RepositoryBranchPair::try_from_str("https://github.com/rust-lang/rust@develop").unwrap();
    let pair4 =
        RepositoryBranchPair::try_from_str("https://github.com/tokio-rs/tokio@main").unwrap();

    assert_eq!(pair1, pair2);
    assert_ne!(pair1, pair3); // Different branch
    assert_ne!(pair1, pair4); // Different repository
}

#[test]
fn test_repository_branch_pair_hash() {
    use std::collections::HashSet;

    let pair1 =
        RepositoryBranchPair::try_from_str("https://github.com/rust-lang/rust@main").unwrap();
    let pair2 =
        RepositoryBranchPair::try_from_str("https://github.com/rust-lang/rust@main").unwrap();
    let pair3 =
        RepositoryBranchPair::try_from_str("https://github.com/tokio-rs/tokio@main").unwrap();

    let mut set = HashSet::new();
    set.insert(pair1.clone());
    set.insert(pair2); // Should not increase size (same as pair1)
    set.insert(pair3);

    assert_eq!(set.len(), 2);
    assert!(set.contains(&pair1));
}

// =============================================================================
// GroupName Tests
// =============================================================================

#[test]
fn test_group_name_creation() {
    let name = GroupName::from("test-group");
    assert_eq!(name.value(), "test-group");
}

#[test]
fn test_group_name_from_str() {
    let name = GroupName::from("my-group");
    assert_eq!(name.value(), "my-group");
}

#[test]
fn test_group_name_equality() {
    let name1 = GroupName::from("test-group");
    let name2 = GroupName::from("test-group");
    let name3 = GroupName::from("other-group");

    assert_eq!(name1, name2);
    assert_ne!(name1, name3);
}

#[test]
fn test_group_name_display() {
    let name = GroupName::from("display-test-group");
    assert_eq!(name.to_string(), "display-test-group");
}

#[test]
fn test_group_name_hash() {
    use std::collections::HashSet;

    let name1 = GroupName::from("test-group");
    let name2 = GroupName::from("test-group");
    let name3 = GroupName::from("other-group");

    let mut set = HashSet::new();
    set.insert(name1.clone());
    set.insert(name2); // Should not increase size (same as name1)
    set.insert(name3);

    assert_eq!(set.len(), 2);
    assert!(set.contains(&name1));
}

#[test]
fn test_group_name_with_special_characters() {
    let special_chars = vec![
        "group-with-dashes",
        "group_with_underscores",
        "group123",
        "group.with.dots",
        "group with spaces",
    ];

    for name_str in special_chars {
        let name = GroupName::from(name_str);
        assert_eq!(name.value(), name_str);
    }
}

// =============================================================================
// Integration Tests
// =============================================================================

#[test]
fn test_repository_branch_pair_integration_with_group_name() {
    // This test demonstrates typical usage where pairs are created and associated with groups
    let group_name = GroupName::from("integration-test-group");

    let pairs = RepositoryBranchPair::try_from_specifiers(&[
        "https://github.com/rust-lang/rust@main".to_string(),
        "https://github.com/tokio-rs/tokio@master".to_string(),
    ])
    .unwrap();

    assert_eq!(group_name.value(), "integration-test-group");
    assert_eq!(pairs.len(), 2);

    // Verify pairs can be used in collections
    use std::collections::HashMap;
    let mut pair_metadata = HashMap::new();
    for (i, pair) in pairs.iter().enumerate() {
        pair_metadata.insert(pair.clone(), format!("Unit {}", i + 1));
    }

    assert_eq!(pair_metadata.len(), 2);
}

#[test]
fn test_repository_branch_pair_serialization_roundtrip() {
    use serde_json;

    let original_pair =
        RepositoryBranchPair::try_from_str("https://github.com/rust-lang/rust@main").unwrap();

    // Test JSON serialization
    let json = serde_json::to_string(&original_pair).unwrap();
    let deserialized_pair: RepositoryBranchPair = serde_json::from_str(&json).unwrap();

    assert_eq!(original_pair, deserialized_pair);
    assert_eq!(original_pair.to_string(), deserialized_pair.to_string());
}

#[test]
fn test_group_name_serialization_roundtrip() {
    use serde_json;

    let original_name = GroupName::from("serialization-test-group");

    // Test JSON serialization
    let json = serde_json::to_string(&original_name).unwrap();
    let deserialized_name: GroupName = serde_json::from_str(&json).unwrap();

    assert_eq!(original_name, deserialized_name);
    assert_eq!(original_name.value(), deserialized_name.value());
}
