use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::github::graphql::graphql_types::comment::CommentsConnection;
use crate::github::graphql::graphql_types::pager::PageInfo;
use crate::github::graphql::graphql_types::timeline::TimelineItemsConnection;
use crate::github::graphql::graphql_types::user::{AssigneesConnection, Author};
use crate::github::graphql::graphql_types::{LabelsConnection, MilestoneNode};
use crate::types::label::Label;
use crate::types::{IssueOrPullrequestId, PullRequest, PullRequestId, PullRequestState, User};

const MERGEABLE_VALUE: &str = "MERGEABLE";
const CONFLICTING_VALUE: &str = "CONFLICTING";

/// GraphQL response structures for Pull Requests API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestsResponse {
    pub repository: PullRequestRepository,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestRepository {
    #[serde(rename = "pullRequests")]
    pub pull_requests: PullRequestsConnection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestsConnection {
    pub nodes: Vec<PullRequestNode>,
    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestNode {
    pub number: i32,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
    pub base_ref_name: Option<String>,
    pub head_ref_name: Option<String>,
    pub mergeable: Option<String>,
    pub merged: Option<bool>,
    #[serde(rename = "mergedAt")]
    pub merged_at: Option<DateTime<Utc>>,
    pub url: String,
    pub author: Option<Author>,
    pub assignees: Option<AssigneesConnection>,
    #[serde(rename = "reviewRequests")]
    pub review_requests: Option<ReviewRequestsConnection>,
    pub labels: Option<LabelsConnection>,
    #[serde(rename = "closedAt")]
    pub closed_at: Option<DateTime<Utc>>,
    pub commits: Option<CommitsConnection>,
    pub additions: Option<i32>,
    pub deletions: Option<i32>,
    #[serde(rename = "changedFiles")]
    pub changed_files: Option<i32>,
    pub milestone: Option<MilestoneNode>,
    pub locked: Option<bool>,
    #[serde(rename = "isDraft")]
    pub is_draft: Option<bool>,
    pub comments: CommentsConnection,
    pub reviews: Option<ReviewsConnection>,
    #[serde(rename = "timelineItems")]
    pub timeline_items: Option<TimelineItemsConnection>,
}

impl TryFrom<(PullRequestNode, crate::types::RepositoryId)> for PullRequest {
    type Error = anyhow::Error;

    fn try_from(
        (pull_request_node, git_repository_id): (PullRequestNode, crate::types::RepositoryId),
    ) -> Result<Self, Self::Error> {
        // Parse assignees
        let assignees = pull_request_node
            .assignees
            .as_ref()
            .map(|assignees| {
                assignees
                    .nodes
                    .iter()
                    .map(|assignee| User::from(assignee.login.clone()))
                    .collect()
            })
            .unwrap_or_default();

        let requested_reviewers: Vec<User> = pull_request_node
            .review_requests
            .as_ref()
            .map(|review_requests| {
                review_requests
                    .nodes
                    .iter()
                    .filter_map(|review_request| {
                        review_request
                            .requested_reviewer
                            .as_ref()
                            .and_then(|reviewer| match reviewer {
                                RequestedReviewer::User { login } => {
                                    Some(User::from(login.clone()))
                                }
                                RequestedReviewer::Team { .. } => None,
                            })
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Parse labels
        let labels = pull_request_node
            .labels
            .as_ref()
            .map(|labels| {
                labels
                    .nodes
                    .iter()
                    .map(|label| Label::from(label.name.clone()))
                    .collect()
            })
            .unwrap_or_default();

        // Parse author
        let author = pull_request_node
            .author
            .map(|author| User::from(author.login));

        // Parse state using strum
        let state = pull_request_node
            .state
            .parse::<PullRequestState>()
            .unwrap_or(PullRequestState::Closed);

        // Parse milestone number
        let milestone_number = pull_request_node
            .milestone
            .as_ref()
            .map(|milestone| milestone.number as u64);

        // Extract linked resources from timeline events (preferred) and fallback to text parsing
        let mut linked_resources =
            if let Some(ref timeline_items) = pull_request_node.timeline_items {
                timeline_items.into()
            } else {
                Vec::new()
            };

        // Fallback: also extract from text content for any missed references
        let mut text_linked_resources = Vec::new();

        // Extract from PR body
        if let Some(ref body) = pull_request_node.body {
            text_linked_resources
                .extend(IssueOrPullrequestId::extract_resource_url_from_text(body));
        }

        // Extract from PR comments
        for comment_node in &pull_request_node.comments.nodes {
            text_linked_resources.extend(IssueOrPullrequestId::extract_resource_url_from_text(
                &comment_node.body,
            ));
        }

        // Merge timeline-based and text-based results, prioritizing timeline data
        for text_resource in text_linked_resources {
            if !linked_resources.contains(&text_resource) {
                linked_resources.push(text_resource);
            }
        }

        // Create GitPullRequest
        let git_pull_request_id =
            PullRequestId::new(git_repository_id, pull_request_node.number as u32);

        // Parse comments from GraphQL response
        let comments: Result<Vec<_>, _> = pull_request_node
            .comments
            .nodes
            .iter()
            .map(|comment_node| crate::types::PullRequestComment::try_from(comment_node.clone()))
            .collect();
        let comments = comments?;

        // Extract reviewers from review data
        let reviewers: Vec<User> = pull_request_node
            .reviews
            .as_ref()
            .map(|reviews| {
                reviews
                    .nodes
                    .iter()
                    .filter_map(|review| review.author.as_ref().map(|author| author.login.clone()))
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .map(User::from)
                    .collect()
            })
            .unwrap_or_default();

        Ok(PullRequest {
            pull_request_id: git_pull_request_id,
            title: pull_request_node.title,
            body: pull_request_node.body,
            state,
            author,
            assignees,
            requested_reviewers,
            reviewers,
            labels,
            head_branch: pull_request_node.head_ref_name.unwrap_or_default(),
            base_branch: pull_request_node.base_ref_name.unwrap_or_default(),
            created_at: pull_request_node.created_at,
            updated_at: pull_request_node.updated_at,
            closed_at: pull_request_node.closed_at,
            merged_at: pull_request_node.merged_at,
            commits_count: pull_request_node
                .commits
                .as_ref()
                .map(|c| c.total_count as u32)
                .unwrap_or(0),
            additions: pull_request_node.additions.unwrap_or(0) as u32,
            deletions: pull_request_node.deletions.unwrap_or(0) as u32,
            changed_files: pull_request_node.changed_files.unwrap_or(0) as u32,
            comments,
            milestone_id: milestone_number,
            draft: pull_request_node.is_draft.unwrap_or(false),
            mergeable: pull_request_node
                .mergeable
                .as_ref()
                .and_then(|s| match s.as_str() {
                    MERGEABLE_VALUE => Some(true),
                    CONFLICTING_VALUE => Some(false),
                    _ => None,
                }),
            linked_resources,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitsConnection {
    #[serde(rename = "totalCount")]
    pub total_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewsConnection {
    pub nodes: Vec<ReviewNode>,
    #[serde(rename = "totalCount")]
    pub total_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRequestsConnection {
    pub nodes: Vec<ReviewRequestNode>,
    #[serde(rename = "totalCount")]
    pub total_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRequestNode {
    #[serde(rename = "requestedReviewer")]
    pub requested_reviewer: Option<RequestedReviewer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "__typename")]
pub enum RequestedReviewer {
    User { login: String },
    Team { name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewNode {
    pub id: String,
    pub state: String,
    pub body: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    pub author: Option<Author>,
    pub url: Option<String>,
}

/// Response structure for multiple pull requests query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiplePullRequestsResponse {
    pub repository: MultiplePullRequestsRepository,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiplePullRequestsRepository {
    #[serde(flatten)]
    pub pull_requests: std::collections::HashMap<String, Option<PullRequestNode>>,
}
