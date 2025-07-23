//! Tests for RepositoryBranchUnit and related types
//!
//! These tests verify the parsing, validation, and functionality of repository branch
//! units, group names, and related type conversions.

use github_insight::types::{
    Branch,
    profile::{GroupName, RepositoryBranchUnit},
    repository::{Owner, RepositoryId, RepositoryName},
};

#[test]
fn test_repository_branch_unit_creation() {
    let repository_id = RepositoryId {
        owner: Owner::from("rust-lang"),
        repository_name: RepositoryName::from("rust"),
    };
    let branch = Branch::new("main");

    let unit = RepositoryBranchUnit::new(repository_id.clone(), branch.clone());

    assert_eq!(unit.repository_id, repository_id);
    assert_eq!(unit.branch, branch);
}

#[test]
fn test_repository_branch_unit_try_from_str_valid_format() {
    let specifier = "https://github.com/rust-lang/rust@main";
    let unit = RepositoryBranchUnit::try_from_str(specifier).unwrap();

    assert_eq!(unit.repository_id.owner.as_str(), "rust-lang");
    assert_eq!(unit.repository_id.repository_name.as_str(), "rust");
    assert_eq!(unit.branch.as_str(), "main");
}

#[test]
fn test_repository_branch_unit_try_from_str_with_different_branch() {
    let specifier = "https://github.com/tokio-rs/tokio@master";
    let unit = RepositoryBranchUnit::try_from_str(specifier).unwrap();

    assert_eq!(unit.repository_id.owner.as_str(), "tokio-rs");
    assert_eq!(unit.repository_id.repository_name.as_str(), "tokio");
    assert_eq!(unit.branch.as_str(), "master");
}

#[test]
fn test_repository_branch_unit_try_from_str_with_feature_branch() {
    let specifier = "https://github.com/serde-rs/serde@feature/async-support";
    let unit = RepositoryBranchUnit::try_from_str(specifier).unwrap();

    assert_eq!(unit.repository_id.owner.as_str(), "serde-rs");
    assert_eq!(unit.repository_id.repository_name.as_str(), "serde");
    assert_eq!(unit.branch.as_str(), "feature/async-support");
}

#[test]
fn test_repository_branch_unit_try_from_str_invalid_format_no_at() {
    let specifier = "https://github.com/rust-lang/rust";
    let result = RepositoryBranchUnit::try_from_str(specifier);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Invalid repository branch specifier format"));
    assert!(error_msg.contains("Expected format: 'repo_url@branch'"));
}

#[test]
fn test_repository_branch_unit_try_from_str_invalid_format_multiple_at() {
    let specifier = "https://github.com/rust-lang/rust@main@dev";
    let result = RepositoryBranchUnit::try_from_str(specifier);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Invalid repository branch specifier format"));
}

#[test]
fn test_repository_branch_unit_try_from_str_empty_repo_url() {
    let specifier = "@main";
    let result = RepositoryBranchUnit::try_from_str(specifier);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Repository URL cannot be empty"));
}

#[test]
fn test_repository_branch_unit_try_from_str_empty_branch() {
    let specifier = "https://github.com/rust-lang/rust@";
    let result = RepositoryBranchUnit::try_from_str(specifier);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Branch name cannot be empty"));
}

#[test]
fn test_repository_branch_unit_try_from_str_whitespace_handling() {
    let specifier = "  https://github.com/rust-lang/rust@main  ";
    let unit = RepositoryBranchUnit::try_from_str(specifier).unwrap();

    assert_eq!(unit.repository_id.owner.as_str(), "rust-lang");
    assert_eq!(unit.repository_id.repository_name.as_str(), "rust");
    assert_eq!(unit.branch.as_str(), "main");
}

#[test]
fn test_repository_branch_unit_try_from_str_whitespace_around_at() {
    let specifier = "https://github.com/rust-lang/rust @ main";
    let unit = RepositoryBranchUnit::try_from_str(specifier).unwrap();

    assert_eq!(unit.repository_id.owner.as_str(), "rust-lang");
    assert_eq!(unit.repository_id.repository_name.as_str(), "rust");
    assert_eq!(unit.branch.as_str(), "main");
}

#[test]
fn test_repository_branch_unit_try_from_specifiers_single() {
    let specifiers = vec!["https://github.com/rust-lang/rust@main".to_string()];
    let units = RepositoryBranchUnit::try_from_specifiers(&specifiers).unwrap();

    assert_eq!(units.len(), 1);
    assert_eq!(units[0].repository_id.owner.as_str(), "rust-lang");
    assert_eq!(units[0].repository_id.repository_name.as_str(), "rust");
    assert_eq!(units[0].branch.as_str(), "main");
}

#[test]
fn test_repository_branch_unit_try_from_specifiers_multiple() {
    let specifiers = vec![
        "https://github.com/rust-lang/rust@main".to_string(),
        "https://github.com/tokio-rs/tokio@master".to_string(),
        "https://github.com/serde-rs/serde@main".to_string(),
    ];
    let units = RepositoryBranchUnit::try_from_specifiers(&specifiers).unwrap();

    assert_eq!(units.len(), 3);

    assert_eq!(units[0].repository_id.owner.as_str(), "rust-lang");
    assert_eq!(units[0].repository_id.repository_name.as_str(), "rust");
    assert_eq!(units[0].branch.as_str(), "main");

    assert_eq!(units[1].repository_id.owner.as_str(), "tokio-rs");
    assert_eq!(units[1].repository_id.repository_name.as_str(), "tokio");
    assert_eq!(units[1].branch.as_str(), "master");

    assert_eq!(units[2].repository_id.owner.as_str(), "serde-rs");
    assert_eq!(units[2].repository_id.repository_name.as_str(), "serde");
    assert_eq!(units[2].branch.as_str(), "main");
}

#[test]
fn test_repository_branch_unit_try_from_specifiers_empty() {
    let specifiers: Vec<String> = vec![];
    let units = RepositoryBranchUnit::try_from_specifiers(&specifiers).unwrap();

    assert_eq!(units.len(), 0);
}

#[test]
fn test_repository_branch_unit_try_from_specifiers_with_invalid() {
    let specifiers = vec![
        "https://github.com/rust-lang/rust@main".to_string(),
        "invalid-format".to_string(),
    ];
    let result = RepositoryBranchUnit::try_from_specifiers(&specifiers);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Invalid repository branch specifier format"));
}

#[test]
fn test_repository_branch_unit_display() {
    let unit =
        RepositoryBranchUnit::try_from_str("https://github.com/rust-lang/rust@main").unwrap();
    let display_str = unit.to_string();

    assert_eq!(display_str, "https://github.com/rust-lang/rust@main");
}

#[test]
fn test_repository_branch_unit_display_different_formats() {
    let test_cases = vec![
        "https://github.com/rust-lang/rust@main",
        "https://github.com/tokio-rs/tokio@master",
        "https://github.com/serde-rs/serde@feature/async-support",
    ];

    for specifier in test_cases {
        let unit = RepositoryBranchUnit::try_from_str(specifier).unwrap();
        assert_eq!(unit.to_string(), specifier);
    }
}

#[test]
fn test_repository_branch_unit_equality() {
    let unit1 =
        RepositoryBranchUnit::try_from_str("https://github.com/rust-lang/rust@main").unwrap();
    let unit2 =
        RepositoryBranchUnit::try_from_str("https://github.com/rust-lang/rust@main").unwrap();
    let unit3 =
        RepositoryBranchUnit::try_from_str("https://github.com/rust-lang/rust@develop").unwrap();
    let unit4 =
        RepositoryBranchUnit::try_from_str("https://github.com/tokio-rs/tokio@main").unwrap();

    assert_eq!(unit1, unit2);
    assert_ne!(unit1, unit3); // Different branch
    assert_ne!(unit1, unit4); // Different repository
}

#[test]
fn test_repository_branch_unit_hash() {
    use std::collections::HashSet;

    let unit1 =
        RepositoryBranchUnit::try_from_str("https://github.com/rust-lang/rust@main").unwrap();
    let unit2 =
        RepositoryBranchUnit::try_from_str("https://github.com/rust-lang/rust@main").unwrap();
    let unit3 =
        RepositoryBranchUnit::try_from_str("https://github.com/tokio-rs/tokio@main").unwrap();

    let mut set = HashSet::new();
    set.insert(unit1.clone());
    set.insert(unit2); // Should not increase size (same as unit1)
    set.insert(unit3);

    assert_eq!(set.len(), 2);
    assert!(set.contains(&unit1));
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
fn test_repository_branch_unit_integration_with_group_name() {
    // This test demonstrates typical usage where units are created and associated with groups
    let group_name = GroupName::from("integration-test-group");

    let units = RepositoryBranchUnit::try_from_specifiers(&[
        "https://github.com/rust-lang/rust@main".to_string(),
        "https://github.com/tokio-rs/tokio@master".to_string(),
    ])
    .unwrap();

    assert_eq!(group_name.value(), "integration-test-group");
    assert_eq!(units.len(), 2);

    // Verify units can be used in collections
    use std::collections::HashMap;
    let mut unit_metadata = HashMap::new();
    for (i, unit) in units.iter().enumerate() {
        unit_metadata.insert(unit.clone(), format!("Unit {}", i + 1));
    }

    assert_eq!(unit_metadata.len(), 2);
}

#[test]
fn test_repository_branch_unit_serialization_roundtrip() {
    use serde_json;

    let original_unit =
        RepositoryBranchUnit::try_from_str("https://github.com/rust-lang/rust@main").unwrap();

    // Test JSON serialization
    let json = serde_json::to_string(&original_unit).unwrap();
    let deserialized_unit: RepositoryBranchUnit = serde_json::from_str(&json).unwrap();

    assert_eq!(original_unit, deserialized_unit);
    assert_eq!(original_unit.to_string(), deserialized_unit.to_string());
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
