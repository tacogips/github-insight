use crate::tools::functions;
use crate::types::ProfileName;
use anyhow::Result;
use rmcp::{Error as McpError, model::*};
use serde_json;

/// List all repository URLs registered in the current profile
///
/// Returns an array of repository URLs for repositories managed by the profile.
pub async fn list_repository_urls_in_current_profile(
    profile_name: &Option<ProfileName>,
) -> Result<CallToolResult, McpError> {
    let profile_name = profile_name.clone().unwrap_or_default().to_string();

    let result = functions::profile::list_repositories(profile_name)
        .await
        .map_err(|e| McpError::internal_error(e, None))?;

    let content = Content::text(serde_json::to_string_pretty(&result).map_err(|e| {
        McpError::internal_error(format!("Failed to serialize result: {}", e), None)
    })?);

    Ok(CallToolResult {
        content: vec![content],
        is_error: Some(false),
    })
}
