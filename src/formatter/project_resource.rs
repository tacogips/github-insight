use crate::types::{ProjectOriginalResource, ProjectResource};

use super::{
    MarkdownContent, TimezoneOffset, format_date_with_timezone_offset,
    format_datetime_with_timezone_offset,
};

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
    content.push_str(&format!("Author: {}\n", project_resource.author));
    content.push_str(&format!("State: {}\n", project_resource.state));
    content.push_str(&format!(
        "Column: {}\n",
        project_resource
            .column_name
            .as_deref()
            .unwrap_or("No Status")
    ));
    content.push_str(&format!(
        "Project Item ID: {}\n",
        project_resource.project_item_id
    ));

    content.push('\n');
    content.push_str("## Due date\n");
    // Display start date
    match project_resource.start_date {
        Some(start) => content.push_str(&format!(
            "- Start Date: {}\n",
            format_date_with_timezone_offset(start, timezone)
        )),
        None => content.push_str("- Start Date: [Empty]\n"),
    }

    // Display end date
    match project_resource.end_date {
        Some(end) => content.push_str(&format!(
            "- End Date: {}\n",
            format_date_with_timezone_offset(end, timezone)
        )),
        None => content.push_str("- End Date: [Empty]\n"),
    }

    content.push('\n');

    // Custom fields
    if !project_resource.custom_field_values.is_empty() {
        content.push_str("## Custom Fields\n");
        for custom_field in &project_resource.custom_field_values {
            content.push_str(&format!("- Field ID: {}\n", custom_field.field_id));
            match &custom_field.value {
                crate::types::project::ProjectFieldValue::Text(text) => {
                    content.push_str(&format!(
                        "- {}: {} (type: Text)\n",
                        custom_field.field_name, text
                    ));
                }
                crate::types::project::ProjectFieldValue::Number(num) => {
                    content.push_str(&format!(
                        "- {}: {} (type: Number)\n",
                        custom_field.field_name, num
                    ));
                }
                crate::types::project::ProjectFieldValue::Date(date) => {
                    content.push_str(&format!(
                        "- {}: {} (type: Date)\n",
                        custom_field.field_name,
                        format_datetime_with_timezone_offset(*date, timezone)
                    ));
                }
                crate::types::project::ProjectFieldValue::SingleSelect(value) => {
                    content.push_str(&format!(
                        "- {}: {} (type: SingleSelect)\n",
                        custom_field.field_name, value
                    ));
                }
                crate::types::project::ProjectFieldValue::MultiSelect(values) => {
                    content.push_str(&format!(
                        "- {}: {} (type: MultiSelect)\n",
                        custom_field.field_name,
                        values.join(", ")
                    ));
                }
            }
            content.push('\n');
        }
    }

    // Original resource reference
    content.push_str("## Original resource\n");
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
    content.push('\n');

    // Assignees
    if !project_resource.assignees.is_empty() {
        content.push_str("## Assignees\n");
        for assignee in &project_resource.assignees {
            content.push_str(&format!("- {}\n", assignee));
        }
        content.push('\n');
    }

    // Timestamps
    content.push_str("## Timestamps\n");
    if let Some(created_at) = project_resource.created_at {
        content.push_str(&format!(
            "- Created: {}\n",
            format_datetime_with_timezone_offset(created_at, timezone)
        ));
    }
    if let Some(updated_at) = project_resource.updated_at {
        content.push_str(&format!(
            "- Updated: {}\n",
            format_datetime_with_timezone_offset(updated_at, timezone)
        ));
    }

    MarkdownContent(content)
}
