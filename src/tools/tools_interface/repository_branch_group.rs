use crate::formatter::{
    TimezoneOffset,
    repository_branch_group::{
        repository_branch_group_list_with_descriptions_markdown,
        repository_branch_group_markdown_with_timezone,
    },
};
use crate::tools::functions;
use crate::types::ProfileName;
use anyhow::Result;
use rmcp::{Error as McpError, model::*};
use serde_json;

/// Register a repository branch group to a profile
///
/// Creates a new repository branch group with branches and optional description.
/// Returns the final group name (auto-generated if not provided) as a JSON string.
pub async fn register_repository_branch_group(
    profile_name: String,
    group_name: Option<String>,
    pairs: Vec<String>,
    description: Option<String>,
) -> Result<CallToolResult, McpError> {
    let final_group_name = functions::profile::register_repository_branch_group_with_description(
        profile_name,
        group_name,
        pairs,
        description,
    )
    .await
    .map_err(|e| McpError::internal_error(e, None))?;

    let content = Content::text(
        serde_json::to_string_pretty(&final_group_name).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize result: {}", e), None)
        })?,
    );

    Ok(CallToolResult {
        content: vec![content],
        is_error: Some(false),
    })
}

/// Remove a repository branch group from a profile
///
/// Completely removes the group and all its branches. Returns the removed group
/// information as JSON.
pub async fn unregister_repository_branch_group(
    profile_name: String,
    group_name: String,
) -> Result<CallToolResult, McpError> {
    let removed_group =
        functions::profile::unregister_repository_branch_group(profile_name, group_name)
            .await
            .map_err(|e| McpError::internal_error(e, None))?;

    let content = Content::text(serde_json::to_string_pretty(&removed_group).map_err(|e| {
        McpError::internal_error(format!("Failed to serialize result: {}", e), None)
    })?);

    Ok(CallToolResult {
        content: vec![content],
        is_error: Some(false),
    })
}

/// Add branches to an existing group
///
/// Allows expanding group membership by adding new branches. Returns success
/// confirmation message upon completion.
pub async fn add_branch_to_branch_group(
    profile_name: String,
    group_name: String,
    branch_specifiers: Vec<String>,
) -> Result<CallToolResult, McpError> {
    functions::profile::add_branch_to_branch_group(profile_name, group_name, branch_specifiers)
        .await
        .map_err(|e| McpError::internal_error(e, None))?;

    let content = Content::text("Branches added successfully".to_string());

    Ok(CallToolResult {
        content: vec![content],
        is_error: Some(false),
    })
}

/// Remove branches from a group
///
/// Allows reducing group membership by removing specific branches. Returns success
/// confirmation message upon completion.
pub async fn remove_branch_from_branch_group(
    profile_name: String,
    group_name: String,
    branch_specifiers: Vec<String>,
) -> Result<CallToolResult, McpError> {
    functions::profile::remove_branch_from_branch_group(
        profile_name,
        group_name,
        branch_specifiers,
    )
    .await
    .map_err(|e| McpError::internal_error(e, None))?;

    let content = Content::text("Branches removed successfully".to_string());

    Ok(CallToolResult {
        content: vec![content],
        is_error: Some(false),
    })
}

/// Rename a repository branch group
///
/// Changes the group's identifier while preserving all branches and metadata.
/// Returns success confirmation message upon completion.
pub async fn rename_repository_branch_group(
    profile_name: String,
    old_name: String,
    new_name: String,
) -> Result<CallToolResult, McpError> {
    functions::profile::rename_repository_branch_group(profile_name, old_name, new_name)
        .await
        .map_err(|e| McpError::internal_error(e, None))?;

    let content = Content::text("Group renamed successfully".to_string());

    Ok(CallToolResult {
        content: vec![content],
        is_error: Some(false),
    })
}

/// List all repository branch groups in a profile
///
/// Shows all groups available for management operations. Returns formatted markdown
/// list showing profile name and all group names.
pub async fn show_repository_branch_groups(
    profile_name: String,
) -> Result<CallToolResult, McpError> {
    let profile_name_str = profile_name.clone();
    let groups = functions::profile::list_repository_branch_groups_with_details(
        &ProfileName::from(profile_name.as_str()),
    )
    .await
    .map_err(|e| McpError::internal_error(e, None))?;

    let formatted =
        repository_branch_group_list_with_descriptions_markdown(&groups, &profile_name_str);
    let content = Content::text(formatted.0);

    Ok(CallToolResult {
        content: vec![content],
        is_error: Some(false),
    })
}

/// Show details of a specific repository branch group
///
/// Returns comprehensive information about the group and all its branches in formatted
/// markdown with group name, creation timestamp, and list of all branches.
pub async fn get_repository_branch_group(
    timezone: &Option<TimezoneOffset>,
    profile_name: String,
    group_name: String,
) -> Result<CallToolResult, McpError> {
    let group = functions::profile::get_repository_branch_group(profile_name, group_name)
        .await
        .map_err(|e| McpError::internal_error(e, None))?;

    let formatted = repository_branch_group_markdown_with_timezone(&group, timezone.as_ref());
    let content = Content::text(formatted.0);

    Ok(CallToolResult {
        content: vec![content],
        is_error: Some(false),
    })
}

/// Remove repository branch groups older than N days
///
/// Useful for cleaning up temporary or outdated groups automatically. Returns JSON
/// array of removed groups with their details.
pub async fn cleanup_repository_branch_groups(
    profile_name: String,
    days: i64,
) -> Result<CallToolResult, McpError> {
    let removed_groups = functions::profile::cleanup_repository_branch_groups(profile_name, days)
        .await
        .map_err(|e| McpError::internal_error(e, None))?;

    let content = Content::text(serde_json::to_string_pretty(&removed_groups).map_err(|e| {
        McpError::internal_error(format!("Failed to serialize result: {}", e), None)
    })?);

    Ok(CallToolResult {
        content: vec![content],
        is_error: Some(false),
    })
}
