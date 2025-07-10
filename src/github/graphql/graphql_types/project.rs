use crate::github::graphql::graphql_types::pager::PageInfo;
use crate::github::graphql::graphql_types::user::{AssigneesConnection, UserNode};
use crate::types::{User, label::Label};
use crate::types::{
    issue::IssueId,
    project::{
        ProjectCustomFieldValue, ProjectFieldValue, ProjectOriginalResource, ProjectResource,
    },
    pull_request::PullRequestId,
    repository::{RepositoryId, RepositoryUrl},
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectNodeIdResponse {
    pub organization: Option<OrganizationNode>,
    pub user: Option<UserNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectResourcesResponse {
    pub organization: Option<OrganizationProjectResponse>,
    pub user: Option<UserProjectResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationProjectResponse {
    #[serde(rename = "projectV2")]
    pub project_v2: Option<ProjectNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProjectResponse {
    #[serde(rename = "projectV2")]
    pub project_v2: Option<ProjectNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectNode {
    pub id: Option<String>,
    pub title: Option<String>,
    pub url: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
    pub fields: Option<FieldsConnection>,
    pub items: Option<ItemsConnection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationNode {
    pub project_v2: Option<ProjectNode>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldsConnection {
    pub nodes: Vec<ProjectField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectItem {
    pub id: String,
    pub content: Option<ProjectItemContent>,
    #[serde(rename = "fieldValues")]
    pub field_values: Option<FieldValuesConnection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "__typename")]
pub enum ProjectItemContent {
    Issue {
        id: Option<String>,
        number: Option<u64>,
        title: Option<String>,
        url: Option<String>,
        state: Option<String>,
        #[serde(rename = "createdAt")]
        created_at: Option<DateTime<Utc>>,
        #[serde(rename = "updatedAt")]
        updated_at: Option<DateTime<Utc>>,
        author: Option<AuthorNode>,
        assignees: Option<AssigneesConnection>,
        labels: Option<LabelsConnection>,
    },
    PullRequest {
        id: Option<String>,
        number: Option<u64>,
        title: Option<String>,
        url: Option<String>,
        state: Option<String>,
        #[serde(rename = "createdAt")]
        created_at: Option<DateTime<Utc>>,
        #[serde(rename = "updatedAt")]
        updated_at: Option<DateTime<Utc>>,
        author: Option<AuthorNode>,
        assignees: Option<AssigneesConnection>,
        labels: Option<LabelsConnection>,
    },
    DraftIssue {
        id: Option<String>,
        title: Option<String>,
        #[serde(rename = "createdAt")]
        created_at: Option<DateTime<Utc>>,
        #[serde(rename = "updatedAt")]
        updated_at: Option<DateTime<Utc>>,
    },
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldValuesConnection {
    pub nodes: Vec<FieldValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "__typename")]
pub enum FieldValue {
    #[serde(rename = "ProjectV2ItemFieldTextValue")]
    Text {
        field: FieldRef,
        text: Option<String>,
    },
    #[serde(rename = "ProjectV2ItemFieldSingleSelectValue")]
    SingleSelect {
        field: FieldRef,
        name: Option<String>,
    },
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "__typename")]
pub enum ProjectField {
    #[serde(rename = "ProjectV2SingleSelectField")]
    SingleSelect {
        id: String,
        name: String,
        options: Option<Vec<SingleSelectOption>>,
    },
    #[serde(rename = "ProjectV2Field")]
    Text { id: String, name: String },
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemsConnection {
    pub nodes: Vec<ProjectItem>,
    #[serde(rename = "pageInfo")]
    pub page_info: Option<PageInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldRef {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleSelectOption {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "__typename")]
pub enum AuthorNode {
    User {
        login: String,
    },
    Organization {
        login: String,
    },
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelsConnection {
    pub nodes: Vec<LabelNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelNode {
    pub name: String,
}

/// Extract author login from AuthorNode
fn extract_author(author: &Option<AuthorNode>) -> User {
    match author {
        Some(AuthorNode::User { login }) => User::from(login.clone()),
        Some(AuthorNode::Organization { login }) => User::from(login.clone()),
        Some(AuthorNode::Other) | None => User::from("".to_string()),
    }
}

/// Extract assignees from AssigneesConnection
fn extract_assignees(assignees: &Option<AssigneesConnection>) -> Vec<User> {
    match assignees {
        Some(connection) => connection
            .nodes
            .iter()
            .map(|node| User::from(node.login.clone()))
            .collect(),
        None => Vec::new(),
    }
}

/// Extract labels from LabelsConnection
fn extract_labels(labels: &Option<LabelsConnection>) -> Vec<Label> {
    match labels {
        Some(connection) => connection
            .nodes
            .iter()
            .map(|node| Label::from(node.name.clone()))
            .collect(),
        None => Vec::new(),
    }
}

impl TryFrom<ProjectItem> for ProjectResource {
    type Error = anyhow::Error;

    fn try_from(project_item: ProjectItem) -> Result<Self, Self::Error> {
        let content = project_item.content.ok_or_else(|| {
            anyhow::anyhow!("Project item has no content - treating as draft issue")
        })?;

        // Extract custom field values
        let mut custom_field_values = Vec::new();
        if let Some(field_values) = project_item.field_values {
            for field_value in field_values.nodes {
                match field_value {
                    FieldValue::Text { field, text } => {
                        if let Some(text_value) = text {
                            custom_field_values.push(ProjectCustomFieldValue {
                                field_id: field.id,
                                field_name: field.name,
                                value: ProjectFieldValue::Text(text_value),
                            });
                        }
                    }
                    FieldValue::SingleSelect { field, name } => {
                        if let Some(select_value) = name {
                            custom_field_values.push(ProjectCustomFieldValue {
                                field_id: field.id,
                                field_name: field.name,
                                value: ProjectFieldValue::SingleSelect(select_value),
                            });
                        }
                    }
                    FieldValue::Other => {
                        // Skip unsupported field value types
                    }
                }
            }
        }

        // Extract column name from field values (look for Status field)
        let column_name = custom_field_values
            .iter()
            .find(|fv| fv.field_name == "Status")
            .and_then(|fv| match &fv.value {
                ProjectFieldValue::Text(text) => Some(text.clone()),
                ProjectFieldValue::SingleSelect(select) => Some(select.clone()),
                _ => None,
            });

        match content {
            ProjectItemContent::Issue {
                number: Some(issue_number),
                title,
                url,
                state,
                created_at,
                updated_at,
                author,
                assignees,
                labels,
                ..
            } => {
                // Extract repository information from URL
                let repository_id = url
                    .as_ref()
                    .and_then(|u| RepositoryId::parse_url(&RepositoryUrl::from(u.as_str())).ok())
                    .ok_or_else(|| {
                        anyhow::anyhow!("Could not extract repository info from issue URL")
                    })?;

                let issue_id = IssueId::new(repository_id, issue_number as u32);

                Ok(ProjectResource {
                    resource_id: project_item.id,
                    title: Some(title.unwrap_or_default()),
                    author: extract_author(&author),
                    assignees: extract_assignees(&assignees),
                    labels: extract_labels(&labels),
                    state: state.unwrap_or_else(|| "open".to_string()),
                    created_at: Some(created_at.unwrap_or_else(Utc::now)),
                    updated_at: Some(updated_at.unwrap_or_else(Utc::now)),
                    column_name,
                    custom_field_values,
                    original_resource: ProjectOriginalResource::Issue(issue_id),
                })
            }
            ProjectItemContent::PullRequest {
                number: Some(pr_number),
                title,
                url,
                state,
                created_at,
                updated_at,
                author,
                assignees,
                labels,
                ..
            } => {
                // Extract repository information from URL
                let repository_id = url
                    .as_ref()
                    .and_then(|u| RepositoryId::parse_url(&RepositoryUrl::from(u.as_str())).ok())
                    .ok_or_else(|| {
                        anyhow::anyhow!("Could not extract repository info from PR URL")
                    })?;

                let pr_id = PullRequestId::new(repository_id, pr_number as u32);

                Ok(ProjectResource {
                    resource_id: project_item.id,
                    title: Some(title.unwrap_or_default()),
                    author: extract_author(&author),
                    assignees: extract_assignees(&assignees),
                    labels: extract_labels(&labels),
                    state: state.unwrap_or_else(|| "open".to_string()),
                    created_at: Some(created_at.unwrap_or_else(Utc::now)),
                    updated_at: Some(updated_at.unwrap_or_else(Utc::now)),
                    column_name,
                    custom_field_values,
                    original_resource: ProjectOriginalResource::PullRequest(pr_id),
                })
            }
            ProjectItemContent::DraftIssue {
                title,
                created_at,
                updated_at,
                ..
            } => Ok(ProjectResource {
                resource_id: project_item.id,
                title: Some(title.unwrap_or_else(|| "Draft Issue".to_string())),
                author: User::from("".to_string()),
                assignees: vec![],
                labels: vec![],
                state: "draft".to_string(),
                created_at: Some(created_at.unwrap_or_else(Utc::now)),
                updated_at: Some(updated_at.unwrap_or_else(Utc::now)),
                column_name,
                custom_field_values,
                original_resource: ProjectOriginalResource::DraftIssue,
            }),
            ProjectItemContent::Other => {
                // For other unsupported content types
                Err(anyhow::anyhow!(
                    "Unsupported project item content type: Other"
                ))
            }
            _ => Err(anyhow::anyhow!("Unsupported project item content type")),
        }
    }
}
