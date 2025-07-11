use crate::types::Issue;

use super::{MarkdownContent, TimezoneOffset, format_datetime_with_timezone_offset};

/// Maximum number of characters to display in the body of an issue in light format
const MAX_BODY_LENGTH: usize = 100;

/// Format an issue into markdown with timezone conversion
pub fn issue_body_markdown_with_timezone(
    issue: &Issue,
    timezone: Option<&TimezoneOffset>,
) -> MarkdownContent {
    let mut content = String::new();

    // Header
    content.push_str(&format!("# ISSUE: {}\n", issue.title));
    content.push_str(&format!("author: {}\n", issue.author));
    content.push_str(&format!("status: {}\n", issue.state));
    content.push_str(&format!("url: {}\n", issue.issue_id.url()));
    content.push_str(&format!(
        "Repository Url: {}\n",
        issue.issue_id.git_repository.url()
    ));

    // Linked resources (Issues and Pull Requests)
    content.push_str("## linked resources \n");
    if !issue.linked_resources.is_empty() {
        for linked in &issue.linked_resources {
            match linked {
                crate::types::IssueOrPullrequestId::IssueId(issue_id) => {
                    content.push_str(&format!("- Issue: {}\n", issue_id.url()));
                }
                crate::types::IssueOrPullrequestId::PullrequestId(pr_id) => {
                    content.push_str(&format!("- PR: {}\n", pr_id.url()));
                }
            }
        }
        content.push('\n');
    }

    // Date information
    content.push_str(&format!(
        "created: {}\n",
        format_datetime_with_timezone_offset(issue.created_at, timezone)
    ));
    content.push_str(&format!(
        "updated: {}\n",
        format_datetime_with_timezone_offset(issue.updated_at, timezone)
    ));
    if let Some(closed_at) = issue.closed_at {
        content.push_str(&format!(
            "closed: {}\n",
            format_datetime_with_timezone_offset(closed_at, timezone)
        ));
    }
    content.push('\n');

    // Body
    content.push_str("## body\n");
    if let Some(body) = &issue.body {
        content.push_str(body);
    }
    content.push_str("\n\n");

    // Labels
    if !issue.labels.is_empty() {
        content.push_str("## labels\n");
        for label in &issue.labels {
            content.push_str(&format!("- {}\n", label));
        }
        content.push('\n');
    }

    // Assignees
    if !issue.assignees.is_empty() {
        content.push_str("## assignee\n");
        for assignee in &issue.assignees {
            content.push_str(&format!("- {}\n", assignee));
        }
        content.push('\n');
    }

    // Comments
    if !issue.comments.is_empty() {
        content.push_str("## comments\n");
        for comment in &issue.comments {
            let author_display = match &comment.author {
                Some(user) => user.as_str().to_string(),
                None => "Unknown ⚠️".to_string(),
            };
            content.push_str(&format!("### author: {}\n", author_display));
            content.push_str(&format!(
                "created: {}\n",
                format_datetime_with_timezone_offset(comment.created_at, timezone)
            ));
            content.push_str(&format!(
                "updated: {}\n",
                format_datetime_with_timezone_offset(comment.updated_at, timezone)
            ));
            content.push_str(&format!("\n{}\n\n", comment.body));
        }
    }

    MarkdownContent(content)
}

pub fn issue_body_markdown_with_timezone_light(
    issue: &Issue,
    _timezone: Option<&TimezoneOffset>,
) -> MarkdownContent {
    let mut content = String::new();

    // Lightweight header - title and status only
    content.push_str(&format!("# {}\n", issue.title));
    content.push_str(&format!("**{}**\n", issue.state));
    content.push_str(&format!("**URL:** {}\n\n", issue.issue_id.url()));

    // Assignees
    if !issue.assignees.is_empty() {
        content.push_str("**Assignees:** ");
        let assignees: Vec<String> = issue.assignees.iter().map(|a| format!("`{}`", a)).collect();
        content.push_str(&assignees.join(" "));
        content.push('\n');
    }

    // Body only if present, truncated to MAX_BODY_LENGTH characters
    if let Some(body) = &issue.body {
        if body.chars().count() > MAX_BODY_LENGTH {
            let truncated: String = body.chars().take(MAX_BODY_LENGTH).collect();
            content.push_str(&truncated);
            content.push_str("...\n\n");
        } else {
            content.push_str(body);
            content.push_str("\n\n");
        }
    }

    // Comment count
    content.push_str(&format!("**Comments:** {}\n", issue.comments_count));

    // Linked resources
    if !issue.linked_resources.is_empty() {
        let urls: Vec<String> = issue
            .linked_resources
            .iter()
            .map(|each| each.url())
            .collect();
        content.push_str(&format!("**Linked:** {}\n", urls.join(",")));
    }

    MarkdownContent(content)
}
