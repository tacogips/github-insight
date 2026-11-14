use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::github::graphql::graphql_types::pager::PageInfo;
use crate::github::graphql::graphql_types::user::Author;
use crate::types::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentsConnection {
    pub nodes: Vec<CommentNode>,
    #[serde(rename = "totalCount")]
    pub total_count: i32,
    #[serde(rename = "pageInfo")]
    pub page_info: Option<PageInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentNode {
    pub id: String,
    pub body: String,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
    pub author: Option<Author>,
    pub url: Option<String>,
}

impl TryFrom<CommentNode> for crate::types::PullRequestComment {
    type Error = anyhow::Error;

    fn try_from(comment_node: CommentNode) -> Result<Self, Self::Error> {
        let body = comment_node.body.clone();
        let author = comment_node
            .author
            .as_ref()
            .map(|a| crate::types::User::from(a.login.clone()));

        // Extract comment ID from GitHub comment URL
        let comment_number = if let Some(ref url) = comment_node.url {
            // Extract ID from GitHub comment URL pattern: .../pull/.../issuecomment-{id}
            if let Some(id_str) = url.split("issuecomment-").last() {
                id_str
                    .parse::<u64>()
                    .map_err(|_| anyhow::anyhow!("Failed to parse comment ID from URL: {}", url))?
            } else {
                return Err(anyhow::anyhow!("Invalid comment URL format: {}", url));
            }
        } else {
            return Err(anyhow::anyhow!("Comment URL is required but missing"));
        };

        Ok(PullRequestComment {
            comment_number,
            body,
            author,
            created_at: comment_node.created_at,
            updated_at: comment_node.updated_at,
        })
    }
}

impl TryFrom<CommentNode> for crate::types::IssueComment {
    type Error = anyhow::Error;

    fn try_from(comment_node: CommentNode) -> Result<Self, Self::Error> {
        let body = comment_node.body.clone();
        let author = comment_node
            .author
            .as_ref()
            .map(|a| crate::types::User::from(a.login.clone()));

        // Extract comment ID from GitHub comment URL
        let comment_number = if let Some(ref url) = comment_node.url {
            // Extract ID from GitHub comment URL pattern: .../issues/.../issuecomment-{id}
            if let Some(id_str) = url.split("issuecomment-").last() {
                id_str
                    .parse::<u64>()
                    .map_err(|_| anyhow::anyhow!("Failed to parse comment ID from URL: {}", url))?
            } else {
                return Err(anyhow::anyhow!("Invalid comment URL format: {}", url));
            }
        } else {
            return Err(anyhow::anyhow!("Comment URL is required but missing"));
        };

        Ok(IssueComment {
            comment_number: crate::types::IssueCommentNumber(comment_number),
            body,
            author,
            created_at: comment_node.created_at,
            updated_at: comment_node.updated_at,
        })
    }
}
