pub mod issue;
pub mod project;
pub mod project_resource;
pub mod pull_request;
pub mod repository;
pub mod repository_branch_group;

use chrono::{DateTime, FixedOffset, Local, Utc};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};

pub use issue::*;
pub use project::*;
pub use project_resource::*;
pub use pull_request::*;
pub use repository::*;
pub use repository_branch_group::*;

/// Common timezone abbreviations with their UTC offsets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display, EnumIter)]
pub enum TimezoneAbbreviation {
    #[strum(serialize = "UTC")]
    Utc,
    #[strum(serialize = "GMT")]
    Gmt,
    #[strum(serialize = "JST")]
    Jst,
    #[strum(serialize = "EST")]
    Est,
    #[strum(serialize = "PST")]
    Pst,
    #[strum(serialize = "PDT")]
    Pdt,
    #[strum(serialize = "BST")]
    Bst,
}

impl TimezoneAbbreviation {
    /// Get the offset hours for this timezone abbreviation
    pub fn offset_hours(&self) -> i32 {
        match self {
            Self::Utc | Self::Gmt => 0,
            Self::Jst => 9,
            Self::Est => -5,
            Self::Pst => -8,
            Self::Pdt => -7,
            Self::Bst => 1,
        }
    }

    /// Get the offset minutes for this timezone abbreviation (usually 0)
    pub fn offset_minutes(&self) -> i32 {
        0 // All supported abbreviations are on hour boundaries
    }

    /// Create a TimezoneOffset from this abbreviation
    pub fn to_timezone_offset(&self) -> TimezoneOffset {
        TimezoneOffset::new(self.offset_hours(), self.offset_minutes(), self.to_string())
    }
}

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
        // First try to parse as a known timezone abbreviation
        if let Ok(tz_abbr) = tz_str.parse::<TimezoneAbbreviation>() {
            return Some(tz_abbr.to_timezone_offset());
        }

        // Handle UTC and GMT as special cases (both map to UTC)
        if tz_str.eq_ignore_ascii_case("UTC") || tz_str.eq_ignore_ascii_case("GMT") {
            return Some(Self::new(0, 0, "UTC".to_string()));
        }

        // Handle offset format strings like "+09:00", "-05:30"
        if tz_str.starts_with('+') || tz_str.starts_with('-') {
            let sign = if tz_str.starts_with('-') { -1 } else { 1 };
            let parts: Vec<&str> = tz_str[1..].split(':').collect();
            if parts.len() == 2 {
                if let (Ok(hours), Ok(minutes)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>())
                {
                    return Some(Self::new(sign * hours, sign * minutes, tz_str.to_string()));
                }
            }
        }

        None
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
