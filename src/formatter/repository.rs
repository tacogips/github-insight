use crate::formatter::{MarkdownContent, TimezoneOffset, format_datetime_with_timezone_offset};
use crate::types::GithubRepository;

// Limit to 10 releases by default
const DEFAULT_RELEASE_LIMIT: usize = 10;
// Limit to 10 milestones by default
const DEFAULT_MILESTONE_LIMIT: usize = 10;

pub fn repository_body_markdown_with_timezone(
    repository: &GithubRepository,
    timezone: Option<&TimezoneOffset>,
    showing_release_limit: Option<usize>,
    showing_milestone_limit: Option<usize>,
) -> MarkdownContent {
    let mut content = String::new();

    // URLs
    content.push_str("## URL\n");
    content.push_str(&format!("{}\n", repository.git_repository_id.url()));
    content.push('\n');

    // Description
    content.push_str("## Description\n");
    if let Some(description) = &repository.description {
        content.push_str(description);
    }
    content.push('\n');

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

        // Sort milestones by due date descending (newest first), then by milestone name
        let mut sorted_milestones = repository.milestones.clone();
        sorted_milestones.sort_by(|a, b| match (a.due_date, b.due_date) {
            (Some(a_due), Some(b_due)) => b_due.cmp(&a_due),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.milestone_name.cmp(&b.milestone_name),
        });

        let showing_milestone_limit = showing_milestone_limit.unwrap_or(DEFAULT_MILESTONE_LIMIT);
        let total_milestones = sorted_milestones.len();
        let display_milestones = if total_milestones > showing_milestone_limit {
            &sorted_milestones[..showing_milestone_limit]
        } else {
            &sorted_milestones
        };

        for milestone in display_milestones {
            let due_date_info = if let Some(due_date) = milestone.due_date {
                format!(
                    " (Due date: {})",
                    format_datetime_with_timezone_offset(due_date, timezone)
                )
            } else {
                String::new()
            };
            content.push_str(&format!(
                "- {} (Milestone number: #{}){}\n",
                milestone.milestone_name, milestone.milestone_number.0, due_date_info
            ));
        }

        // Show omitted milestones message if applicable
        if total_milestones > showing_milestone_limit {
            let omitted_count = total_milestones - showing_milestone_limit;
            content.push_str(&format!(
                "\n*{} older milestones omitted (showing {} most recent)*\n",
                omitted_count, showing_milestone_limit
            ));
        }
    }

    // Releases (if any)
    if !repository.releases.is_empty() {
        content.push_str("\n## Releases\n");

        // Sort releases by newest first (created_at descending)
        let mut sorted_releases = repository.releases.clone();
        sorted_releases.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let showing_release_limit = showing_release_limit.unwrap_or(DEFAULT_RELEASE_LIMIT);
        let total_releases = sorted_releases.len();
        let display_releases = if total_releases > showing_release_limit {
            &sorted_releases[..showing_release_limit]
        } else {
            &sorted_releases
        };

        for release in display_releases {
            let mut release_info = format!("- **{}** ({})", release.name, release.tag_name);

            if release.is_prerelease {
                release_info.push_str(" [Pre-release]");
            }
            if release.is_draft {
                release_info.push_str(" [Draft]");
            }

            content.push_str(&format!("{}\n", release_info));

            if let Some(description) = &release.description {
                if !description.is_empty() {
                    content.push_str(&format!("  - Description: {}\n", description));
                }
            }

            content.push_str(&format!(
                "  - Created: {}\n",
                format_datetime_with_timezone_offset(release.created_at, timezone)
            ));

            if let Some(published_at) = release.published_at {
                content.push_str(&format!(
                    "  - Published: {}\n",
                    format_datetime_with_timezone_offset(published_at, timezone)
                ));
            }

            if let Some(author) = &release.author {
                content.push_str(&format!("  - Author: {}\n", author.as_str()));
            }

            content.push_str(&format!("  - URL: {}\n", release.url));
        }

        // Show omitted releases message if applicable
        if total_releases > showing_release_limit {
            let omitted_count = total_releases - showing_release_limit;
            content.push_str(&format!(
                "\n*{} older releases omitted (showing {} most recent)*\n",
                omitted_count, showing_release_limit
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
