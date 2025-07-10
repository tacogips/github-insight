use crate::types::{ProjectOriginalResource, ProjectResource};

use super::{MarkdownContent, TimezoneOffset, format_datetime_with_timezone_offset};

/// Format a project resource into markdown without timezone conversion
pub fn project_resource_body_markdown(project_resource: &ProjectResource) -> MarkdownContent {
    project_resource_body_markdown_with_timezone(project_resource, None)
}

pub fn project_resource_body_markdown_with_timezone(
    project_resource: &ProjectResource,
    timezone: Option<&TimezoneOffset>,
) -> MarkdownContent {
    let mut content = String::new();

    let title = project_resource.title.as_deref().unwrap_or("(No title)");
    content.push_str(&format!("# {}\n", title));
    content.push_str(&format!("author: {}\n", project_resource.author));
    content.push_str(&format!("state: {}\n", project_resource.state));
    content.push('\n');

    // Original resource reference
    content.push_str("## original resource\n");
    match &project_resource.original_resource {
        ProjectOriginalResource::Issue(issue_id) => {
            content.push_str("- Type: Issue\n");
            content.push_str(&format!("- Url: {}\n", issue_id.url()));
        }
        ProjectOriginalResource::PullRequest(pr_id) => {
            content.push_str("- Type: Pull Request\n");
            content.push_str(&format!("- Url: {}\n", pr_id.url()));
        }
        ProjectOriginalResource::DraftIssue => {
            content.push_str("- Type: Draft Issue (project-only)\n");
        }
    }

    // Column information
    if let Some(column_name) = &project_resource.column_name {
        content.push_str(&format!("column: {}\n", column_name));
    }
    content.push('\n');

    //NO labels for now
    //// Labels
    //if !project_resource.labels.is_empty() {
    //    content.push_str("## labels\n");
    //    for label in &project_resource.labels {
    //        content.push_str(&format!("- {}\n", label));
    //    }
    //    content.push('\n');
    //}

    // Assignees
    if !project_resource.assignees.is_empty() {
        content.push_str("## assignees\n");
        for assignee in &project_resource.assignees {
            content.push_str(&format!("- {}\n", assignee));
        }
        content.push('\n');
    }

    // Timestamps
    content.push_str("## timestamps\n");
    if let Some(updated_at) = project_resource.updated_at {
        content.push_str(&format!(
            "- Updated: {}\n",
            format_datetime_with_timezone_offset(updated_at, timezone)
        ));
    }

    MarkdownContent(content)
}
