use crate::formatter::{MarkdownContent, TimezoneOffset, format_datetime_with_timezone_offset};
use crate::types::GithubRepository;

pub fn repository_body_markdown(repository: &GithubRepository) -> MarkdownContent {
    repository_body_markdown_with_timezone(repository, None)
}

pub fn repository_body_markdown_with_timezone(
    repository: &GithubRepository,
    timezone: Option<&TimezoneOffset>,
) -> MarkdownContent {
    let mut content = String::new();

    // URLs
    content.push_str("## URL\n");
    content.push_str(&format!("{}\n", repository.git_repository_id.url()));
    content.push_str("\n");

    // Description
    content.push_str("## Description\n");
    if let Some(description) = &repository.description {
        content.push_str(description);
    }
    content.push_str("\n");

    // Repository Information
    content.push_str("## Default Branch\n");
    if let Some(default_branch) = &repository.default_branch {
        content.push_str(&format!("{}\n", default_branch.as_str()));
    }

    // Users (if any)
    if !repository.users.is_empty() {
        content.push_str("\n## Mentionable Users\n");
        for user in &repository.users {
            content.push_str(&format!("- {}\n", user.as_str()));
        }
    }

    // Labels (if any)
    if !repository.labels.is_empty() {
        content.push_str("\n## Labels\n");
        for label in &repository.labels {
            content.push_str(&format!("- {}\n", label.name()));
        }
    }

    // Milestones (if any)
    if !repository.milestones.is_empty() {
        content.push_str("\n## Milestones\n");
        for milestone in &repository.milestones {
            let due_date_info = if let Some(due_date) = milestone.due_date {
                format!(
                    " (Due date: {})",
                    format_datetime_with_timezone_offset(due_date, timezone)
                )
            } else {
                String::new()
            };
            content.push_str(&format!(
                "- {} (Milestone ID: {}){}\n",
                milestone.milestone_name, milestone.milestone_id, due_date_info
            ));
        }
    }

    // Timestamps
    content.push_str("## Timestamps\n");
    content.push_str(&format!(
        "- Created: {}\n",
        format_datetime_with_timezone_offset(repository.created_at, timezone)
    ));
    content.push_str(&format!(
        "- Updated: {}\n",
        format_datetime_with_timezone_offset(repository.updated_at, timezone)
    ));

    MarkdownContent(content)
}
