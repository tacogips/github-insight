//! Project domain types and URL parsing
//!
//! This module contains the Project domain types with comprehensive URL parsing
//! capabilities. Following domain-driven design principles, all project-specific
//! URL parsing logic is contained within this module.

use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use regex::Regex;
use schemars::JsonSchema;

use crate::types::label::Label;
use crate::types::user::User;
use serde::{Deserialize, Serialize};

use crate::types::{issue::IssueId, pull_request::PullRequestId, repository::Owner};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectUrl(pub String);

impl std::fmt::Display for ProjectUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

static PROJECT_URL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:https?://)?github\.com/(orgs|users)/([^/]+)/projects/(\d+)")
        .expect("Failed to compile project URL regex")
});

/// Project type to distinguish between user and organization projects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum ProjectType {
    /// User project (github.com/users/owner/projects/123)
    User,
    /// Organization project (github.com/orgs/owner/projects/123)
    Organization,
}

/// Project number wrapper for type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct ProjectNumber(pub u64);

impl ProjectNumber {
    /// Create new project number
    pub fn new(number: u64) -> Self {
        Self(number)
    }

    /// Get the numeric value
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for ProjectNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Strong-typed project identifier with URL parsing capabilities.
///
/// This struct encapsulates all project identification logic and URL parsing
/// specific to projects. Following domain-driven design, all project URL
/// parsing logic is self-contained within this domain.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct ProjectId {
    pub owner: Owner,
    pub number: ProjectNumber,
    pub project_type: ProjectType,
}

impl ProjectId {
    /// Create new project identifier
    pub fn new(owner: Owner, number: ProjectNumber, project_type: ProjectType) -> Self {
        Self {
            owner,
            number,
            project_type,
        }
    }
    pub fn url(&self) -> String {
        let path_type = match self.project_type {
            ProjectType::Organization => "orgs",
            ProjectType::User => "users",
        };
        format!(
            "https://github.com/{}/{}/projects/{}",
            path_type, self.owner, self.number
        )
    }

    /// Parse GitHub project URL to extract owner, project number, and project type
    ///
    /// Domain-specific URL parsing moved from utils to maintain domain boundaries.
    /// Supports both user and organization project URLs.
    pub fn parse_url(url: &ProjectUrl) -> Result<(String, u64, ProjectType), String> {
        let url = url.0.to_string();
        let url = url.trim_end_matches('/');

        // Parse GitHub project URL patterns:
        // https://github.com/orgs/owner/projects/123
        // https://github.com/users/owner/projects/123
        if let Some(captures) = PROJECT_URL_REGEX.captures(url) {
            let project_type = match captures.get(1).unwrap().as_str() {
                "orgs" => ProjectType::Organization,
                "users" => ProjectType::User,
                _ => return Err("Invalid project type".to_string()),
            };
            let owner = captures.get(2).unwrap().as_str().to_string();
            let number = captures
                .get(3)
                .unwrap()
                .as_str()
                .parse::<u64>()
                .map_err(|_| "Invalid project number")?;

            return Ok((owner, number, project_type));
        }

        Err(format!("Invalid GitHub project URL format: {}", url))
    }

    /// Returns the owner part of the project
    pub fn owner(&self) -> &Owner {
        &self.owner
    }

    /// Returns the project number
    pub fn project_number(&self) -> ProjectNumber {
        self.number
    }

    /// Returns the project type
    pub fn project_type(&self) -> ProjectType {
        self.project_type
    }
}

impl std::fmt::Display for ProjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url())
    }
}

/// Git project with resources and custom fields.
///
/// Contains comprehensive project information including custom fields,
/// project items, and resource management capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub project_id: ProjectId,
    pub title: String,
    pub description: Option<String>,
    pub state: ProjectState,
    pub visibility: ProjectVisibility,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub creator: String,
    pub url: String,
    pub resources: Vec<ProjectResource>,
    pub custom_fields: Vec<ProjectCustomField>,
}

/// Represents the state of a GitHub project
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProjectState {
    /// Project is open and active
    Open,
    /// Project is closed
    Closed,
}

/// Represents the visibility of a GitHub project
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProjectVisibility {
    /// Project is public
    Public,
    /// Project is private
    Private,
}

impl Project {
    /// Create new project with basic metadata
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        project_id: ProjectId,
        title: String,
        description: Option<String>,
        state: ProjectState,
        visibility: ProjectVisibility,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        closed_at: Option<DateTime<Utc>>,
        creator: String,
        resources: Vec<ProjectResource>,
    ) -> Self {
        Self {
            url: project_id.url(),
            project_id,
            title,
            description,
            state,
            visibility,
            created_at,
            updated_at,
            closed_at,
            creator,
            resources,
            custom_fields: Vec::new(),
        }
    }
}

/// Individual project item/resource within a GitHub project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectResource {
    pub project_item_id: ProjectItemId,
    pub title: Option<String>,
    pub author: User,
    pub assignees: Vec<User>,
    pub labels: Vec<Label>,
    pub state: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub column_name: Option<String>,
    pub custom_field_values: Vec<ProjectCustomFieldValue>,
    /// Reference to the original issue or PR
    pub original_resource: ProjectOriginalResource,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

/// Type of resource in a project
/// Reference to the original resource (issue or PR)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectOriginalResource {
    /// Reference to an issue
    Issue(IssueId),
    /// Reference to a pull request
    PullRequest(PullRequestId),
    /// Draft issue exists only in project
    DraftIssue,
}

/// Custom field definition for a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCustomField {
    pub field_id: String,
    pub field_name: String,
    pub field_type: ProjectCustomFieldType,
    pub options: Vec<String>,
}

/// Type of custom field in a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectCustomFieldType {
    /// Text field
    Text,
    /// Number field
    Number,
    /// Date field
    Date,
    /// Single select field
    SingleSelect,
    /// Multi select field
    MultiSelect,
}

/// Value of a custom field for a specific resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCustomFieldValue {
    pub field_id: ProjectFieldId,
    pub field_name: ProjectFieldName,
    pub value: ProjectFieldValue,
}

/// Actual value of a custom field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectFieldType {
    /// Text value
    Text,
    /// Number value
    Number,
    /// Date value
    Date,
    /// Single select value
    SingleSelect,
    /// Multi select values
    MultiSelect,
}

/// Actual value of a custom field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectFieldValue {
    /// Text value
    Text(String),
    /// Number value
    Number(f64),
    /// Date value
    Date(DateTime<Utc>),
    /// Single select value
    SingleSelect(String),
    /// Multi select values
    MultiSelect(Vec<String>),
}

impl ProjectResource {
    /// Create new project resource
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        project_item_id: ProjectItemId,
        title: String,
        author: String,
        assignees: Vec<String>,
        labels: Vec<String>,
        state: String,
        column_name: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        original_resource: ProjectOriginalResource,
    ) -> Self {
        Self {
            project_item_id,
            title: Some(title),
            author: User::from(author),
            assignees: assignees.into_iter().map(User::from).collect(),
            labels: labels.into_iter().map(Label::from).collect(),
            state,
            created_at: Some(created_at),
            updated_at: Some(updated_at),
            column_name,
            custom_field_values: Vec::new(),
            original_resource,
            start_date: None,
            end_date: None,
        }
    }

    /// Get the original issue ID if this resource is an issue
    pub fn as_issue_id(&self) -> Option<&IssueId> {
        match &self.original_resource {
            ProjectOriginalResource::Issue(issue_id) => Some(issue_id),
            _ => None,
        }
    }

    /// Get the original pull request ID if this resource is a PR
    pub fn as_pull_request_id(&self) -> Option<&PullRequestId> {
        match &self.original_resource {
            ProjectOriginalResource::PullRequest(pr_id) => Some(pr_id),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectItemId(pub String);

impl std::fmt::Display for ProjectItemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFieldId(pub String);

impl std::fmt::Display for ProjectFieldId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFieldName(pub String);

impl std::fmt::Display for ProjectFieldName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ProjectFieldName {
    pub fn eq_ignore_ascii_case(&self, other: &str) -> bool {
        self.0.eq_ignore_ascii_case(other)
    }
}
