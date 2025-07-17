use crate::formatter::{MarkdownContent, TimezoneOffset, format_datetime_with_timezone_offset};
use crate::types::Project;

pub fn project_body_markdown(project: &Project) -> MarkdownContent {
    project_body_markdown_with_timezone(project, None)
}

pub fn project_body_markdown_with_timezone(
    project: &Project,
    timezone: Option<&TimezoneOffset>,
) -> MarkdownContent {
    let mut content = String::new();

    // Header
    content.push_str(&format!("# PROJECT: {}\n", project.title));
    content.push_str(&format!("project_id: {}\n", project.project_id));
    content.push_str(&format!("url: {}\n\n", project.project_id.url()));

    // Description
    content.push_str("## description\n");
    if let Some(description) = &project.description {
        content.push_str(description);
    } else {
        content.push_str("(No description provided)");
    }
    content.push_str("\n\n");

    // Metadata
    content.push_str("## metadata\n");
    content.push_str(&format!(
        "- Created: {}\n",
        format_datetime_with_timezone_offset(project.created_at, timezone)
    ));
    content.push_str(&format!(
        "- Updated: {}\n",
        format_datetime_with_timezone_offset(project.updated_at, timezone)
    ));
    content.push_str("- Total resources: (fetch resources separately)\n");
    content.push('\n');

    MarkdownContent(content)
}
