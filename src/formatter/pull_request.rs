use crate::types::PullRequest;

use super::{MarkdownContent, TimezoneOffset, format_datetime_with_timezone_offset};

/// Maximum number of characters to display in the body of a pull request in light format
const MAX_BODY_LENGTH: usize = 100;

/// Format a pull request into markdown with timezone conversion
pub fn pull_request_body_markdown_with_timezone(
    pr: &PullRequest,
    timezone: Option<&TimezoneOffset>,
) -> MarkdownContent {
    let mut content = String::new();

    // Header
    content.push_str(&format!("# PR: {}\n", pr.title));
    let author_display = match &pr.author {
        Some(user) => user.as_str().to_string(),
        None => "Unknown ⚠️".to_string(),
    };
    content.push_str(&format!("author: {}\n", author_display));
    content.push_str(&format!("status: {}\n", pr.state));
    content.push_str(&format!("url: {}\n", pr.pull_request_id.url()));
    content.push_str(&format!(
        "Repository Url: {}\n",
        pr.pull_request_id.git_repository.url()
    ));

    // Date information
    content.push_str(&format!(
        "created: {}\n",
        format_datetime_with_timezone_offset(pr.created_at, timezone)
    ));
    content.push_str(&format!(
        "updated: {}\n",
        format_datetime_with_timezone_offset(pr.updated_at, timezone)
    ));
    if let Some(closed_at) = pr.closed_at {
        content.push_str(&format!(
            "closed: {}\n",
            format_datetime_with_timezone_offset(closed_at, timezone)
        ));
    }
    if let Some(merged_at) = pr.merged_at {
        content.push_str(&format!(
            "merged: {}\n",
            format_datetime_with_timezone_offset(merged_at, timezone)
        ));
    }
    content.push('\n');

    // Assignees
    if !pr.assignees.is_empty() {
        content.push_str("## assignee\n");
        for assignee in &pr.assignees {
            content.push_str(&format!("- {}\n", assignee));
        }
        content.push('\n');
    }

    // Labels
    if !pr.labels.is_empty() {
        content.push_str("## labels\n");
        for label in &pr.labels {
            content.push_str(&format!("- {}\n", label));
        }
        content.push('\n');
    }

    // Branch info (HIGH priority)
    if !pr.head_branch.is_empty() && !pr.base_branch.is_empty() {
        content.push_str("## branch info\n");
        content.push_str(&format!("- Source: {}\n", pr.head_branch));
        content.push_str(&format!("- Target: {}\n", pr.base_branch));
        content.push('\n');
    }

    // Reviewers (HIGH priority)
    if !pr.reviewers.is_empty() {
        content.push_str("## reviewers\n");
        content.push_str(&format!(
            "- Reviewed by: {}\n",
            pr.reviewers
                .iter()
                .map(|u| u.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
        content.push('\n');
    }

    // Stats (HIGH priority)
    if pr.additions > 0 || pr.deletions > 0 || pr.changed_files > 0 || pr.commits_count > 0 {
        content.push_str("## stats\n");
        content.push_str(&format!(
            "- Changes: +{} -{} files:{} commits:{}\n",
            pr.additions, pr.deletions, pr.changed_files, pr.commits_count
        ));
        content.push('\n');
    }

    // Flags (HIGH priority)
    if pr.draft {
        content.push_str("## flags\n");
        if pr.draft {
            content.push_str("- Status: DRAFT\n");
        }
        content.push('\n');
    }

    // Milestone (LOW priority)
    if let Some(milestone_id) = &pr.milestone_id {
        content.push_str("## milestone\n");
        content.push_str(&format!("- {}\n", milestone_id));
        content.push('\n');
    }

    // Body
    content.push_str("## body\n");
    if let Some(body) = &pr.body {
        content.push_str(body);
    }
    content.push_str("\n\n");

    // Comments
    content.push_str("## comments\n");
    if !pr.comments.is_empty() {
        for comment in &pr.comments {
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
    } else {
        content.push_str("(No comments)\n\n");
    }

    // Linked resources (Issues and Pull Requests)
    content.push_str("## linked resources \n");
    if !pr.linked_resources.is_empty() {
        for linked in &pr.linked_resources {
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

    MarkdownContent(content)
}

pub fn pull_request_body_markdown_with_timezone_light(
    pr: &PullRequest,
    _timezone: Option<&TimezoneOffset>,
) -> MarkdownContent {
    let mut content = String::new();

    // Lightweight header - title and status only
    content.push_str(&format!("# {}\n", pr.title));
    content.push_str(&format!("**{}**\n", pr.state));
    content.push_str(&format!("**URL:** {}\n\n", pr.pull_request_id.url()));
    // Author
    if let Some(author) = &pr.author {
        content.push_str(&format!("**Author:** `{}`\n", author));
    }

    // Body only if present, truncated to MAX_BODY_LENGTH characters
    if let Some(body) = &pr.body {
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
    content.push_str(&format!("**Comments:** {}\n", pr.comments.len()));

    // Linked resources
    if !pr.linked_resources.is_empty() {
        let urls: Vec<String> = pr.linked_resources.iter().map(|each| each.url()).collect();
        content.push_str(&format!("**Linked:** {}\n", urls.join(",")));
    }

    MarkdownContent(content)
}
