pub mod issue;
pub mod project_resource;
pub mod pull_request;
pub mod repository;

use crate::types::Project;
use chrono::{DateTime, FixedOffset, Local, Utc};
use serde::{Deserialize, Serialize};

pub use issue::*;
pub use project_resource::*;
pub use pull_request::*;
pub use repository::*;

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct MarkdownContent(pub String);

/// Custom timezone offset implementation to replace chrono-tz.
///
/// This struct provides timezone offset functionality without depending on the `chrono-tz` crate,
/// which was causing severe memory issues during compilation and testing (consuming over 62GB RAM
/// during `cargo test` execution due to extensive timezone data loading).
///
/// This lightweight implementation supports:
/// - Common timezone abbreviations (UTC, JST, EST, PST, PDT, BST, GMT)
/// - Offset format strings like "+09:00", "-05:30"
/// - Conversion to chrono's FixedOffset for datetime calculations
///
/// Note: This implementation does not handle Daylight Saving Time (DST) transitions automatically.
/// Users must specify the correct timezone abbreviation (e.g., "EST" vs "EDT") for their use case.
///
/// # Example
/// ```
/// use github_insight::formatter::TimezoneOffset;
///
/// // Parse timezone from string
/// let jst = TimezoneOffset::parse("JST").unwrap();
/// let custom = TimezoneOffset::parse("+09:00").unwrap();
///
/// // Both represent the same offset
/// assert_eq!(jst.offset_seconds, custom.offset_seconds);
/// ```
#[derive(Debug, Clone)]
pub struct TimezoneOffset {
    /// Offset from UTC in seconds (positive for east, negative for west)
    pub offset_seconds: i32,
    /// Human-readable timezone name or offset string
    pub name: String,
}

impl TimezoneOffset {
    /// Create a new timezone offset from hours and minutes
    pub fn new(hours: i32, minutes: i32, name: String) -> Self {
        Self {
            offset_seconds: hours * 3600 + minutes * 60,
            name,
        }
    }

    /// Create a timezone offset from the local system timezone
    pub fn from_local() -> Self {
        let local_offset = Local::now().offset().local_minus_utc();
        let hours = local_offset / 3600;
        let minutes = (local_offset % 3600) / 60;

        let name = if local_offset >= 0 {
            format!("+{:02}:{:02}", hours, minutes)
        } else {
            format!("-{:02}:{:02}", hours.abs(), minutes.abs())
        };

        Self {
            offset_seconds: local_offset,
            name,
        }
    }

    /// Parse timezone offset from string (e.g., "+09:00", "-05:30", "UTC")
    pub fn parse(tz_str: &str) -> Option<Self> {
        match tz_str {
            "UTC" | "GMT" => Some(Self::new(0, 0, "UTC".to_string())),
            "JST" => Some(Self::new(9, 0, "JST".to_string())),
            "EST" => Some(Self::new(-5, 0, "EST".to_string())),
            "PST" => Some(Self::new(-8, 0, "PST".to_string())),
            "PDT" => Some(Self::new(-7, 0, "PDT".to_string())),
            "BST" => Some(Self::new(1, 0, "BST".to_string())),
            s if s.starts_with('+') || s.starts_with('-') => {
                let sign = if s.starts_with('-') { -1 } else { 1 };
                let parts: Vec<&str> = s[1..].split(':').collect();
                if parts.len() == 2 {
                    if let (Ok(hours), Ok(minutes)) =
                        (parts[0].parse::<i32>(), parts[1].parse::<i32>())
                    {
                        Some(Self::new(sign * hours, sign * minutes, s.to_string()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Convert to chrono FixedOffset
    pub fn to_fixed_offset(&self) -> FixedOffset {
        FixedOffset::east_opt(self.offset_seconds).unwrap_or(FixedOffset::east_opt(0).unwrap())
    }
}

impl std::fmt::Display for TimezoneOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Format a UTC datetime with the specified timezone offset.
/// If timezone is None, defaults to UTC.
pub fn format_datetime_with_timezone_offset(
    dt: DateTime<Utc>,
    timezone: Option<&TimezoneOffset>,
) -> String {
    match timezone {
        Some(tz) => {
            let local_dt = dt.with_timezone(&tz.to_fixed_offset());
            local_dt
                .format(&format!("%Y-%m-%d %H:%M:%S {}", tz.name))
                .to_string()
        }
        None => dt.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
    }
}

/// Format a UTC date with the specified timezone offset (date only, no time).
/// If timezone is None, defaults to UTC.
pub fn format_date_with_timezone_offset(
    dt: DateTime<Utc>,
    timezone: Option<&TimezoneOffset>,
) -> String {
    match timezone {
        Some(tz) => {
            let local_dt = dt.with_timezone(&tz.to_fixed_offset());
            local_dt
                .format(&format!("%Y-%m-%d {}", tz.name))
                .to_string()
        }
        None => dt.format("%Y-%m-%d UTC").to_string(),
    }
}

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
    content.push_str(&format!("creator: {}\n", project.creator));
    content.push_str(&format!("project_id: {}\n", project.project_id));
    content.push_str(&format!("url: {}\n\n", project.url));

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
    content.push_str(&format!("- Total resources: {}\n", project.resources.len()));
    content.push('\n');

    // Resources
    if !project.resources.is_empty() {
        content.push_str("## resources\n");
        // TODO: Update when resource_id type is changed from String and fields are updated
        for resource in &project.resources {
            content.push_str(&format!(
                "- **Resource**: {} - Column: {:?}\n",
                resource.project_item_id, resource.column_name
            ));
            content.push_str(&format!("  - State: {}\n", resource.state));
            content.push_str(&format!(
                "  - Created: {}\n",
                format_datetime_with_timezone_offset(
                    resource.created_at.unwrap_or_else(Utc::now),
                    timezone
                )
            ));
            content.push_str(&format!(
                "  - Updated: {}\n\n",
                format_datetime_with_timezone_offset(
                    resource.updated_at.unwrap_or_else(Utc::now),
                    timezone
                )
            ));
        }
    }

    MarkdownContent(content)
}

pub fn project_resources_list_markdown(
    project: &Project,
    limit: usize,
    timezone: Option<&TimezoneOffset>,
) -> MarkdownContent {
    let mut content = String::new();

    content.push_str(&format!("# Project Resources: {}\n\n", project.title));

    if let Some(desc) = &project.description {
        content.push_str(&format!("{}\n\n", desc));
    }

    content.push_str(&format!("**Project URL:** {}\n\n", project.url));
    content.push_str(&format!(
        "**Total Resources:** {}\n\n",
        project.resources.len()
    ));

    // Column names are not available in the current Project struct

    content.push_str("## Resources\n\n");

    // TODO: Update when resource_id type is changed from String and fields are updated
    let display_resources = project.resources.iter().take(limit);
    for resource in display_resources {
        content.push_str(&format!("### Resource - {}\n\n", resource.project_item_id));
        content.push_str(&format!("- **Column:** {:?}\n", resource.column_name));
        content.push_str(&format!("- **State:** {}\n", resource.state));
        content.push_str(&format!(
            "- **Created:** {}\n",
            format_datetime_with_timezone_offset(
                resource.created_at.unwrap_or_else(Utc::now),
                timezone
            )
        ));
        content.push_str(&format!(
            "- **Updated:** {}\n\n",
            format_datetime_with_timezone_offset(
                resource.updated_at.unwrap_or_else(Utc::now),
                timezone
            )
        ));
    }

    if project.resources.len() > limit {
        content.push_str(&format!(
            "*Showing {} of {} resources*\n",
            limit,
            project.resources.len()
        ));
    }

    MarkdownContent(content)
}
