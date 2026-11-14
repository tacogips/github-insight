use github_insight::formatter::repository_branch_group::*;
use github_insight::types::repository::{Owner, RepositoryName};
use github_insight::types::{
    Branch, GroupName, RepositoryBranchGroup, RepositoryBranchPair, RepositoryId,
};

fn create_test_pair() -> RepositoryBranchPair {
    RepositoryBranchPair::new(
        RepositoryId {
            owner: Owner::from("test-owner"),
            repository_name: RepositoryName::from("test-repo"),
        },
        Branch::new("main"),
    )
}

fn create_test_group() -> RepositoryBranchGroup {
    let pair1 = create_test_pair();
    let mut pair2 = create_test_pair();
    pair2.repository_id.repository_name = RepositoryName::from("another-repo");
    pair2.branch = Branch::new("develop");

    RepositoryBranchGroup::new(Some(GroupName::from("test-group")), vec![pair1, pair2])
}

fn create_test_group_with_description() -> RepositoryBranchGroup {
    let pair1 = create_test_pair();
    let mut pair2 = create_test_pair();
    pair2.repository_id.repository_name = RepositoryName::from("another-repo");
    pair2.branch = Branch::new("develop");

    RepositoryBranchGroup::new_with_description(
        Some(GroupName::from("test-group-with-desc")),
        vec![pair1, pair2],
        Some("Test group description".to_string()),
    )
}

#[test]
fn test_repository_branch_group_list_markdown() {
    let groups = vec![GroupName::from("group1"), GroupName::from("group2")];
    let result = repository_branch_group_list_markdown(&groups, "test-dummy-profile");

    assert!(
        result
            .0
            .contains("Repository branch groups in profile 'test-dummy-profile':")
    );
    assert!(result.0.contains("group1"));
    assert!(result.0.contains("group2"));
}

#[test]
fn test_repository_branch_group_list_markdown_empty() {
    let groups = vec![];
    let result = repository_branch_group_list_markdown(&groups, "test-dummy-profile");

    assert!(
        result
            .0
            .contains("No repository branch groups found in profile 'test-dummy-profile'")
    );
}

#[test]
fn test_repository_branch_group_markdown() {
    let group = create_test_group();
    let result = repository_branch_group_markdown_with_timezone(&group, None);

    assert!(result.0.contains("**test-group** (created:"));
    assert!(
        result
            .0
            .contains("https://github.com/test-owner/test-repo | branch:main")
    );
    assert!(
        result
            .0
            .contains("https://github.com/test-owner/another-repo | branch:develop")
    );
}

#[test]
fn test_repository_branch_group_markdown_light() {
    let group = create_test_group();
    let result = repository_branch_group_markdown_with_timezone_light(&group, None);

    assert!(result.0.contains("**test-group** (created:"));
    assert!(
        result
            .0
            .contains("https://github.com/test-owner/test-repo | branch:main")
    );
    assert!(
        result
            .0
            .contains("https://github.com/test-owner/another-repo | branch:develop")
    );
}

#[test]
fn test_repository_branch_group_json() {
    let group = create_test_group();
    let result = repository_branch_group_json(&group);

    assert!(result.is_ok());
    let json_str = result.unwrap();
    assert!(json_str.contains("test-group"));
    assert!(json_str.contains("test-owner"));
    assert!(json_str.contains("main"));
    assert!(json_str.contains("develop"));
}

#[test]
fn test_repository_branch_group_list_json() {
    let groups = vec![GroupName::from("group1"), GroupName::from("group2")];
    let result = repository_branch_group_list_json(&groups);

    assert!(result.is_ok());
    let json_str = result.unwrap();
    assert!(json_str.contains("group1"));
    assert!(json_str.contains("group2"));
}

#[test]
fn test_repository_branch_pair_markdown() {
    let pair = create_test_pair();
    let result = repository_branch_pair_markdown(&pair);

    assert_eq!(
        result.0,
        "https://github.com/test-owner/test-repo | branch:main"
    );
}

#[test]
fn test_multiple_groups_markdown() {
    let group1 = create_test_group();
    let mut group2 = create_test_group();
    group2.name = GroupName::from("second-group");

    let groups = vec![group1, group2];
    let result = repository_branch_groups_markdown_with_timezone(&groups, None);

    assert!(result.0.contains("**test-group** (created:"));
    assert!(result.0.contains("**second-group** (created:"));
    assert!(!result.0.contains("---")); // No separator in compact format
}

#[test]
fn test_repository_branch_group_with_description_markdown() {
    let group = create_test_group_with_description();
    let result = repository_branch_group_markdown_with_timezone(&group, None);

    assert!(result.0.contains("**test-group-with-desc** (created:"));
    assert!(result.0.contains("*Description:* Test group description"));
    assert!(
        result
            .0
            .contains("https://github.com/test-owner/test-repo | branch:main")
    );
    assert!(
        result
            .0
            .contains("https://github.com/test-owner/another-repo | branch:develop")
    );
}

#[test]
fn test_repository_branch_group_with_description_light_markdown() {
    let group = create_test_group_with_description();
    let result = repository_branch_group_markdown_with_timezone_light(&group, None);

    assert!(result.0.contains("**test-group-with-desc** (created:"));
    assert!(result.0.contains("*Description:* Test group description"));
    assert!(
        result
            .0
            .contains("https://github.com/test-owner/test-repo | branch:main")
    );
    assert!(
        result
            .0
            .contains("https://github.com/test-owner/another-repo | branch:develop")
    );
}

#[test]
fn test_repository_branch_group_without_description_markdown() {
    let group = create_test_group();
    let result = repository_branch_group_markdown_with_timezone(&group, None);

    assert!(result.0.contains("**test-group** (created:"));
    assert!(!result.0.contains("*Description:"));
    assert!(
        result
            .0
            .contains("https://github.com/test-owner/test-repo | branch:main")
    );
    assert!(
        result
            .0
            .contains("https://github.com/test-owner/another-repo | branch:develop")
    );
}

#[test]
fn test_repository_branch_group_description_json() {
    let group = create_test_group_with_description();
    let result = repository_branch_group_json(&group);

    assert!(result.is_ok());
    let json_str = result.unwrap();
    assert!(json_str.contains("test-group-with-desc"));
    assert!(json_str.contains("Test group description"));
    assert!(json_str.contains("test-owner"));
    assert!(json_str.contains("main"));
    assert!(json_str.contains("develop"));
}

#[test]
fn test_repository_branch_group_list_with_descriptions_markdown() {
    let group1 = create_test_group();
    let group2 = create_test_group_with_description();
    let groups = vec![group1, group2];
    let result =
        repository_branch_group_list_with_descriptions_markdown(&groups, "test-dummy-profile");

    assert!(
        result
            .0
            .contains("Repository branch groups in profile 'test-dummy-profile':")
    );
    assert!(result.0.contains("  - test-group"));
    assert!(
        result
            .0
            .contains("  - test-group-with-desc - Test group description")
    );
}

#[test]
fn test_repository_branch_group_list_with_descriptions_markdown_empty() {
    let groups = vec![];
    let result =
        repository_branch_group_list_with_descriptions_markdown(&groups, "test-dummy-profile");

    assert!(
        result
            .0
            .contains("No repository branch groups found in profile 'test-dummy-profile'")
    );
}

#[test]
fn test_repository_branch_group_list_with_descriptions_markdown_no_descriptions() {
    let group1 = create_test_group();
    let mut group2 = create_test_group();
    group2.name = GroupName::from("test-group-2");
    let groups = vec![group1, group2];
    let result =
        repository_branch_group_list_with_descriptions_markdown(&groups, "test-dummy-profile");

    assert!(
        result
            .0
            .contains("Repository branch groups in profile 'test-dummy-profile':")
    );
    assert!(result.0.contains("  - test-group"));
    assert!(result.0.contains("  - test-group-2"));
    // Should not contain description text when no descriptions are present
    assert!(!result.0.contains("Test group description"));
}
