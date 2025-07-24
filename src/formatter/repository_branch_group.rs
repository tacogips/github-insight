//! Repository branch group formatting functionality
//!
//! This module provides formatting capabilities for repository branch groups,
//! supporting both markdown and JSON output formats with timezone-aware datetime display.

use crate::types::{GroupName, RepositoryBranchGroup, RepositoryBranchPair};

use super::{MarkdownContent, TimezoneOffset, format_datetime_with_timezone_offset};

/// Format a list of repository branch group names into markdown
pub fn repository_branch_group_list_markdown(
    groups: &[GroupName],
    profile_name: &str,
) -> MarkdownContent {
    let mut content = String::new();

    if groups.is_empty() {
        content.push_str(&format!(
            "No repository branch groups found in profile '{}'",
            profile_name
        ));
    } else {
        content.push_str(&format!(
            "Repository branch groups in profile '{}':",
            profile_name
        ));
        for group_name in groups {
            content.push_str(&format!("\n  - {}", group_name));
        }
    }

    MarkdownContent(content)
}

/// Format a repository branch group into detailed markdown with timezone conversion
pub fn repository_branch_group_markdown_with_timezone(
    group: &RepositoryBranchGroup,
    timezone: Option<&TimezoneOffset>,
) -> MarkdownContent {
    let mut content = String::new();

    // Header with created_at timestamp
    content.push_str(&format!(
        "**{}** (created: {})",
        group.name,
        format_datetime_with_timezone_offset(group.created_at, timezone)
    ));

    // Repository branch pairs (format: repository_url | branch:branch_name)
    if !group.pairs.is_empty() {
        for pair in &group.pairs {
            content.push_str(&format!(
                "\n- {} | branch:{}",
                pair.repository_id.url(),
                pair.branch.as_str()
            ));
        }
    }

    MarkdownContent(content)
}

/// Format a repository branch group into lightweight markdown with timezone conversion
pub fn repository_branch_group_markdown_with_timezone_light(
    group: &RepositoryBranchGroup,
    timezone: Option<&TimezoneOffset>,
) -> MarkdownContent {
    let mut content = String::new();

    // Header with created_at timestamp
    content.push_str(&format!(
        "**{}** (created: {})",
        group.name,
        format_datetime_with_timezone_offset(group.created_at, timezone)
    ));

    // Pairs (format: repository_url | branch:branch_name)
    if !group.pairs.is_empty() {
        for pair in &group.pairs {
            content.push_str(&format!(
                "\n- {} | branch:{}",
                pair.repository_id.url(),
                pair.branch.as_str()
            ));
        }
    }

    MarkdownContent(content)
}

/// Format a repository branch group into JSON format
pub fn repository_branch_group_json(
    group: &RepositoryBranchGroup,
) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(group)
}

/// Format a list of repository branch group names into JSON format
pub fn repository_branch_group_list_json(
    groups: &[GroupName],
) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(groups)
}

/// Format repository branch pair into markdown string
pub fn repository_branch_pair_markdown(pair: &RepositoryBranchPair) -> MarkdownContent {
    let content = format!(
        "{} | branch:{}",
        pair.repository_id.url(),
        pair.branch.as_str()
    );
    MarkdownContent(content)
}

/// Format multiple repository branch groups into markdown with timezone conversion
pub fn repository_branch_groups_markdown_with_timezone(
    groups: &[RepositoryBranchGroup],
    timezone: Option<&TimezoneOffset>,
) -> MarkdownContent {
    let mut content = String::new();

    if groups.is_empty() {
        content.push_str("No repository branch groups found.\n");
    } else {
        for (i, group) in groups.iter().enumerate() {
            if i > 0 {
                content.push_str("\n\n");
            }
            let group_markdown = repository_branch_group_markdown_with_timezone(group, timezone);
            content.push_str(&group_markdown.0);
        }
    }

    MarkdownContent(content)
}

/// Format multiple repository branch groups into lightweight markdown with timezone conversion  
pub fn repository_branch_groups_markdown_with_timezone_light(
    groups: &[RepositoryBranchGroup],
    timezone: Option<&TimezoneOffset>,
) -> MarkdownContent {
    let mut content = String::new();

    if groups.is_empty() {
        content.push_str("No repository branch groups found.\n");
    } else {
        for (i, group) in groups.iter().enumerate() {
            if i > 0 {
                content.push_str("\n\n");
            }
            let group_markdown =
                repository_branch_group_markdown_with_timezone_light(group, timezone);
            content.push_str(&group_markdown.0);
        }
    }

    MarkdownContent(content)
}
