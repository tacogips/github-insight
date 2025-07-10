use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::github::graphql::graphql_types::comment::CommentsConnection;
use crate::github::graphql::graphql_types::pager::PageInfo;
use crate::github::graphql::graphql_types::repository::Repository;
use crate::github::graphql::graphql_types::timeline::TimelineItemsConnection;
use crate::github::graphql::graphql_types::user::{AssigneesConnection, Author};
use crate::github::graphql::graphql_types::{LabelsConnection, MilestoneNode};
use crate::types::{Issue, RepositoryId, User};

// Constants for GraphQL API values
const STATE_OPEN: &str = "OPEN";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueRepository {
    pub issues: IssuesConnection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuesConnection {
    pub nodes: Vec<IssueNode>,
    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueNode {
    pub number: i32,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
    #[serde(rename = "closedAt")]
    pub closed_at: Option<DateTime<Utc>>,
    pub url: String,
    pub comments: CommentsConnection,
    pub labels: Option<LabelsConnection>,
    pub assignees: Option<AssigneesConnection>,
    pub author: Option<Author>,
    pub milestone: Option<MilestoneNode>,
    pub locked: Option<bool>,
    #[serde(rename = "timelineItems")]
    pub timeline_items: Option<TimelineItemsConnection>,
    pub repository: Repository,
}

/// GraphQL response structures for Issues API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuesResponse {
    pub repository: IssueRepository,
}

impl TryFrom<IssueNode> for crate::types::Issue {
    type Error = anyhow::Error;

    fn try_from(issue_node: IssueNode) -> Result<Self, Self::Error> {
        use crate::types::{IssueId, IssueState};

        // Parse assignees
        let assignees = issue_node
            .assignees
            .as_ref()
            .map(|assignees| {
                assignees
                    .nodes
                    .iter()
                    .map(|assignee| assignee.login.clone())
                    .collect()
            })
            .unwrap_or_default();

        // Parse labels
        let labels = issue_node
            .labels
            .as_ref()
            .map(|labels| {
                labels
                    .nodes
                    .iter()
                    .map(|label| label.name.clone())
                    .collect()
            })
            .unwrap_or_default();

        // Parse author
        let author = issue_node
            .author
            .as_ref()
            .map(|author| User::from(author.login.clone()));

        // Parse state
        let state = match issue_node.state.as_str() {
            STATE_OPEN => IssueState::Open,
            _ => IssueState::Closed,
        };

        // Parse milestone number
        let milestone_number = issue_node
            .milestone
            .as_ref()
            .map(|milestone| milestone.number as u64);

        // Create RepositoryId from the issue's repository field
        let git_repository = RepositoryId::new(
            issue_node.repository.owner.login.clone(),
            issue_node.repository.name.clone(),
        );

        // Create GitIssue
        let issue_id = IssueId::new(git_repository, issue_node.number as u32);

        // For now, return empty comments - we'll implement comment parsing separately
        let comments = vec![];

        // Parse relative issue or pull request IDs from timeline items
        let relative_issue_or_pull_requests = issue_node
            .timeline_items
            .as_ref()
            .map(|timeline_items| timeline_items.into())
            .unwrap_or_default();

        Ok(Issue {
            issue_id,
            title: issue_node.title,
            body: issue_node.body,
            state,
            author: author
                .map(|u| u.as_str().to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
            assignees,
            labels,
            created_at: issue_node.created_at,
            updated_at: issue_node.updated_at,
            closed_at: issue_node.closed_at,
            comments_count: issue_node.comments.total_count as u32,
            comments,
            milestone_id: milestone_number,
            locked: issue_node.locked.unwrap_or(false),
            linked_resources: relative_issue_or_pull_requests,
        })
    }
}

/// Response structure for multiple issues query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultipleIssuesResponse {
    pub repository: MultipleIssuesRepository,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultipleIssuesRepository {
    #[serde(flatten)]
    pub issues: std::collections::HashMap<String, Option<IssueNode>>,
}
