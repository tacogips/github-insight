use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::github::graphql::graphql_types::pager::PageInfo;
use crate::github::graphql::graphql_types::repository::{Repository, RepositoryOwner};
use crate::types::{IssueId, IssueOrPullrequestId, PullRequestId, RepositoryId};

// Constants for GraphQL API values
const TYPENAME_CROSS_REFERENCED_EVENT: &str = "CrossReferencedEvent";
const TYPENAME_CONNECTED_EVENT: &str = "ConnectedEvent";
const TYPENAME_DISCONNECTED_EVENT: &str = "DisconnectedEvent";
const TYPENAME_ISSUE: &str = "Issue";
const TYPENAME_PULL_REQUEST: &str = "PullRequest";

/// Timeline event structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineItemsConnection {
    pub nodes: Vec<TimelineItem>,
    #[serde(rename = "pageInfo")]
    pub page_info: Option<PageInfo>,
}

impl From<&TimelineItemsConnection> for Vec<IssueOrPullrequestId> {
    fn from(timeline_items: &TimelineItemsConnection) -> Self {
        let mut resources = Vec::new();
        let mut to_remove = HashSet::<IssueOrPullrequestId>::new();

        for item in &timeline_items.nodes {
            match item {
                TimelineItem::CrossReferenced {
                    source: Some(source),
                    ..
                } => {
                    if let Some(resource) = source.clone().into() {
                        resources.push(resource);
                    }
                }
                TimelineItem::Connected {
                    subject: Some(subject),
                    ..
                } => {
                    if let Some(resource) = subject.clone().into() {
                        resources.push(resource);
                    }
                }
                TimelineItem::Disconnected {
                    subject: Some(subject),
                    ..
                } => {
                    if let Some(resource) = subject.clone().into() {
                        to_remove.insert(resource);
                    }
                }
                _ => {}
            }
        }

        // Remove disconnected items in a single pass
        resources.retain(|r| !to_remove.contains(r));
        resources
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum TimelineItem {
    CrossReferenced {
        created_at: DateTime<Utc>,
        source: Option<CrossReferenceSource>,
        will_close_target: Option<bool>,
    },
    Connected {
        created_at: DateTime<Utc>,
        subject: Option<ConnectedSubject>,
    },
    Disconnected {
        created_at: DateTime<Utc>,
        subject: Option<ConnectedSubject>,
    },
    Other,
}

impl<'de> Deserialize<'de> for TimelineItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{MapAccess, Visitor};
        use std::fmt;

        struct TimelineItemVisitor;

        impl<'de> Visitor<'de> for TimelineItemVisitor {
            type Value = TimelineItem;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a timeline item object")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut typename: Option<String> = None;
                let mut created_at: Option<DateTime<Utc>> = None;
                let mut source: Option<CrossReferenceSource> = None;
                let mut subject: Option<ConnectedSubject> = None;
                let mut will_close_target: Option<bool> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "__typename" => {
                            typename = Some(map.next_value()?);
                        }
                        "createdAt" => {
                            created_at = Some(map.next_value()?);
                        }
                        "source" => {
                            source = map.next_value()?;
                        }
                        "subject" => {
                            subject = map.next_value()?;
                        }
                        "willCloseTarget" => {
                            will_close_target = map.next_value()?;
                        }
                        _ => {
                            // Ignore unknown fields
                            map.next_value::<serde_json::Value>()?;
                        }
                    }
                }

                let created_at = created_at.unwrap_or_else(Utc::now);

                match typename.as_deref() {
                    Some(TYPENAME_CROSS_REFERENCED_EVENT) => Ok(TimelineItem::CrossReferenced {
                        created_at,
                        source,
                        will_close_target,
                    }),
                    Some(TYPENAME_CONNECTED_EVENT) => Ok(TimelineItem::Connected {
                        created_at,
                        subject,
                    }),
                    Some(TYPENAME_DISCONNECTED_EVENT) => Ok(TimelineItem::Disconnected {
                        created_at,
                        subject,
                    }),
                    _ => Ok(TimelineItem::Other),
                }
            }
        }

        deserializer.deserialize_map(TimelineItemVisitor)
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum CrossReferenceSource {
    Issue {
        number: i32,
        title: String,
        url: String,
        state: String,
        repository: Repository,
    },
    PullRequest {
        number: i32,
        title: String,
        url: String,
        state: String,
        repository: Repository,
    },
    Other,
}

impl From<CrossReferenceSource> for Option<IssueOrPullrequestId> {
    fn from(source: CrossReferenceSource) -> Self {
        match source {
            CrossReferenceSource::Issue {
                number, repository, ..
            } => {
                let repo_id = RepositoryId::new(repository.owner.login, repository.name);
                let git_issue_id = IssueId::new(repo_id, number as u32);
                Some(IssueOrPullrequestId::IssueId(git_issue_id))
            }
            CrossReferenceSource::PullRequest {
                number, repository, ..
            } => {
                let repo_id = RepositoryId::new(repository.owner.login, repository.name);
                let git_pr_id = PullRequestId::new(repo_id, number as u32);
                Some(IssueOrPullrequestId::PullrequestId(git_pr_id))
            }
            CrossReferenceSource::Other => None,
        }
    }
}

impl<'de> Deserialize<'de> for CrossReferenceSource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{MapAccess, Visitor};
        use std::fmt;

        struct CrossReferenceSourceVisitor;

        impl<'de> Visitor<'de> for CrossReferenceSourceVisitor {
            type Value = CrossReferenceSource;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a cross reference source object")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut typename: Option<String> = None;
                let mut number: Option<i32> = None;
                let mut title: Option<String> = None;
                let mut url: Option<String> = None;
                let mut state: Option<String> = None;
                let mut repository: Option<Repository> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "__typename" => {
                            typename = Some(map.next_value()?);
                        }
                        "number" => {
                            number = Some(map.next_value()?);
                        }
                        "title" => {
                            title = Some(map.next_value()?);
                        }
                        "url" => {
                            url = Some(map.next_value()?);
                        }
                        "state" => {
                            state = Some(map.next_value()?);
                        }
                        "repository" => {
                            repository = Some(map.next_value()?);
                        }
                        _ => {
                            // Ignore unknown fields
                            map.next_value::<serde_json::Value>()?;
                        }
                    }
                }

                match typename.as_deref() {
                    Some(TYPENAME_ISSUE) => Ok(CrossReferenceSource::Issue {
                        number: number.unwrap_or(0),
                        title: title.unwrap_or_default(),
                        url: url.unwrap_or_default(),
                        state: state.unwrap_or_default(),
                        repository: repository.unwrap_or_else(|| Repository {
                            owner: RepositoryOwner {
                                login: String::new(),
                            },
                            name: String::new(),
                        }),
                    }),
                    Some(TYPENAME_PULL_REQUEST) => Ok(CrossReferenceSource::PullRequest {
                        number: number.unwrap_or(0),
                        title: title.unwrap_or_default(),
                        url: url.unwrap_or_default(),
                        state: state.unwrap_or_default(),
                        repository: repository.unwrap_or_else(|| Repository {
                            owner: RepositoryOwner {
                                login: String::new(),
                            },
                            name: String::new(),
                        }),
                    }),
                    _ => Ok(CrossReferenceSource::Other),
                }
            }
        }

        deserializer.deserialize_map(CrossReferenceSourceVisitor)
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum ConnectedSubject {
    Issue {
        number: i32,
        title: String,
        url: String,
        state: String,
        repository: Repository,
    },
    PullRequest {
        number: i32,
        title: String,
        url: String,
        state: String,
        repository: Repository,
    },
    Other,
}
impl<'de> Deserialize<'de> for ConnectedSubject {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{MapAccess, Visitor};
        use std::fmt;

        struct ConnectedSubjectVisitor;

        impl<'de> Visitor<'de> for ConnectedSubjectVisitor {
            type Value = ConnectedSubject;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a connected subject object")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut typename: Option<String> = None;
                let mut number: Option<i32> = None;
                let mut title: Option<String> = None;
                let mut url: Option<String> = None;
                let mut state: Option<String> = None;
                let mut repository: Option<Repository> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "__typename" => {
                            typename = Some(map.next_value()?);
                        }
                        "number" => {
                            number = Some(map.next_value()?);
                        }
                        "title" => {
                            title = Some(map.next_value()?);
                        }
                        "url" => {
                            url = Some(map.next_value()?);
                        }
                        "state" => {
                            state = Some(map.next_value()?);
                        }
                        "repository" => {
                            repository = Some(map.next_value()?);
                        }
                        _ => {
                            // Ignore unknown fields
                            map.next_value::<serde_json::Value>()?;
                        }
                    }
                }

                match typename.as_deref() {
                    Some(TYPENAME_ISSUE) => Ok(ConnectedSubject::Issue {
                        number: number.unwrap_or(0),
                        title: title.unwrap_or_default(),
                        url: url.unwrap_or_default(),
                        state: state.unwrap_or_default(),
                        repository: repository.unwrap_or_else(|| Repository {
                            owner: RepositoryOwner {
                                login: String::new(),
                            },
                            name: String::new(),
                        }),
                    }),
                    Some(TYPENAME_PULL_REQUEST) => Ok(ConnectedSubject::PullRequest {
                        number: number.unwrap_or(0),
                        title: title.unwrap_or_default(),
                        url: url.unwrap_or_default(),
                        state: state.unwrap_or_default(),
                        repository: repository.unwrap_or_else(|| Repository {
                            owner: RepositoryOwner {
                                login: String::new(),
                            },
                            name: String::new(),
                        }),
                    }),
                    _ => Ok(ConnectedSubject::Other),
                }
            }
        }

        deserializer.deserialize_map(ConnectedSubjectVisitor)
    }
}

impl From<ConnectedSubject> for Option<IssueOrPullrequestId> {
    fn from(subject: ConnectedSubject) -> Self {
        match subject {
            ConnectedSubject::Issue {
                number, repository, ..
            } => {
                let repo_id = RepositoryId::new(repository.owner.login, repository.name);
                let git_issue_id = IssueId::new(repo_id, number as u32);
                Some(IssueOrPullrequestId::IssueId(git_issue_id))
            }
            ConnectedSubject::PullRequest {
                number, repository, ..
            } => {
                let repo_id = RepositoryId::new(repository.owner.login, repository.name);
                let git_pr_id = PullRequestId::new(repo_id, number as u32);
                Some(IssueOrPullrequestId::PullrequestId(git_pr_id))
            }
            ConnectedSubject::Other => None,
        }
    }
}
