use anyhow::Result;
use futures::stream::{self, StreamExt};
use std::collections::BTreeMap;

use crate::github::GitHubClient;
use crate::types::{
    GithubRepository, Issue, IssueNumber, Project, ProjectId, ProjectResource, PullRequest,
    PullRequestNumber, RepositoryId,
};

/// Coordinates batch fetching of multiple resources
pub struct MultiResourceFetcher {
    github_client: GitHubClient,
}

impl MultiResourceFetcher {
    /// Creates a new MultiResourceFetcher instance
    pub fn new(github_client: GitHubClient) -> Self {
        Self { github_client }
    }

    /// Fetches multiple issues by repository
    ///
    /// # Arguments
    ///
    /// * `issue_ids` - Vec of (repo_id, issue_number) tuples
    ///
    /// # Returns
    ///
    /// Returns a Vec of issues
    pub async fn fetch_issues(
        &self,
        issue_ids_of_repositories: Vec<(RepositoryId, Vec<IssueNumber>)>,
    ) -> Result<BTreeMap<RepositoryId, Vec<Issue>>> {
        // Fetch issues from all repositories concurrently
        let fetch_futures =
            issue_ids_of_repositories
                .into_iter()
                .map(|(repo_id, issue_numbers)| {
                    let github_client = self.github_client.clone();

                    async move {
                        match github_client
                            .fetch_multiple_issues_by_numbers(repo_id.clone(), &issue_numbers)
                            .await
                        {
                            Ok(issues) => Ok((repo_id, issues)),
                            Err(e) => {
                                tracing::warn!("Failed to fetch issues from {}: {}", repo_id, e);
                                Err(e)
                            }
                        }
                    }
                });

        let results: Vec<Result<(RepositoryId, Vec<Issue>)>> = stream::iter(fetch_futures)
            .buffer_unordered(10) // Process up to 10 repositories concurrently
            .collect()
            .await;

        let issues_by_repo: BTreeMap<RepositoryId, Vec<Issue>> = results
            .into_iter()
            .filter_map(|result| result.ok())
            .collect();

        Ok(issues_by_repo)
    }

    /// Fetches multiple pull requests by repository
    ///
    /// # Arguments
    ///
    /// * `pr_ids` - Vec of (repo_id, pr_number) tuples
    ///
    /// # Returns
    ///
    /// Returns a Vec of pull requests
    pub async fn fetch_pull_requests(
        &self,
        pr_numbers_of_repositories: Vec<(RepositoryId, Vec<PullRequestNumber>)>,
    ) -> Result<BTreeMap<RepositoryId, Vec<PullRequest>>> {
        // Fetch PRs from all repositories concurrently
        let fetch_futures = pr_numbers_of_repositories.into_iter().map(|(repo_id, pr_numbers)| {
            let github_client = self.github_client.clone();

            async move {
                match github_client
                    .fetch_multiple_pull_requests_by_numbers(
                        repo_id.clone(),
                        &pr_numbers,
                        Some(
                            crate::github::graphql::pull_request::PullRequestQueryLimitSize::default(),
                        ),
                    )
                    .await
                {
                    Ok(prs) => Ok((repo_id, prs)),
                    Err(e) => {
                        tracing::warn!("Failed to fetch PRs from {}: {}", repo_id, e);
                        Err(e)
                    }
                }
            }
        });

        let results: Vec<Result<(RepositoryId, Vec<PullRequest>)>> = stream::iter(fetch_futures)
            .buffer_unordered(10) // Process up to 10 repositories concurrently
            .collect()
            .await;

        let prs_by_repo: BTreeMap<RepositoryId, Vec<PullRequest>> = results
            .into_iter()
            .filter_map(|result| result.ok())
            .collect();

        Ok(prs_by_repo)
    }

    /// Fetches all resources (issues, pull requests, and draft issues) from a GitHub project
    ///
    /// # Arguments
    ///
    /// * `project_id` - The project identifier containing owner, number, and project type
    ///
    /// # Returns
    ///
    /// Returns a Vec of project resources with full metadata including custom fields
    pub async fn fetch_project_resources(
        &self,
        project_id: ProjectId,
    ) -> Result<Vec<ProjectResource>> {
        self.github_client
            .fetch_all_project_resources(project_id)
            .await
    }

    /// Fetches a single repository by its identifier
    ///
    /// # Arguments
    ///
    /// * `repository_id` - The repository identifier containing owner and repository name
    ///
    /// # Returns
    ///
    /// Returns a GithubRepository with complete repository information
    pub async fn fetch_repository(&self, repository_id: RepositoryId) -> Result<GithubRepository> {
        self.github_client.fetch_repository(repository_id).await
    }

    /// Fetches a single project by its identifier
    ///
    /// # Arguments
    ///
    /// * `project_id` - The project identifier containing owner, number, and project type
    ///
    /// # Returns
    ///
    /// Returns a Project with complete project information
    pub async fn fetch_project(&self, project_id: ProjectId) -> Result<Project> {
        self.github_client.fetch_project(project_id).await
    }

    /// Fetches pull request diffs by repository
    ///
    /// # Arguments
    ///
    /// * `pr_numbers_of_repositories` - Vec of (repo_id, pr_number) tuples
    ///
    /// # Returns
    ///
    /// Returns a BTreeMap of repository IDs to vectors of (PR number, diff) tuples
    pub async fn fetch_pull_request_diffs(
        &self,
        pr_numbers_of_repositories: Vec<(RepositoryId, Vec<PullRequestNumber>)>,
    ) -> Result<BTreeMap<RepositoryId, Vec<(PullRequestNumber, String)>>> {
        // Fetch diffs from all repositories concurrently
        let fetch_futures = pr_numbers_of_repositories
            .into_iter()
            .map(|(repo_id, pr_numbers)| {
                let github_client = self.github_client.clone();

                async move {
                    let mut repo_diffs = Vec::new();

                    // Fetch each PR diff sequentially to avoid overwhelming the API
                    for pr_number in pr_numbers {
                        match github_client
                            .fetch_pull_request_diff(repo_id.clone(), pr_number)
                            .await
                        {
                            Ok(diff) => {
                                repo_diffs.push((pr_number, diff));
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "Failed to fetch diff for PR #{} from {}: {}",
                                    pr_number.value(),
                                    repo_id,
                                    e
                                );
                                // Continue to next PR instead of failing completely
                            }
                        }
                    }

                    Ok::<_, anyhow::Error>((repo_id, repo_diffs))
                }
            });

        let results: Vec<Result<(RepositoryId, Vec<(PullRequestNumber, String)>)>> =
            stream::iter(fetch_futures)
                .buffer_unordered(10) // Process up to 10 repositories concurrently
                .collect()
                .await;

        let diffs_by_repo: BTreeMap<RepositoryId, Vec<(PullRequestNumber, String)>> = results
            .into_iter()
            .filter_map(|result| result.ok())
            .collect();

        Ok(diffs_by_repo)
    }

    pub async fn fetch_pull_request_files_stats(
        &self,
        pr_numbers_of_repositories: Vec<(RepositoryId, Vec<PullRequestNumber>)>,
    ) -> Result<BTreeMap<RepositoryId, Vec<(PullRequestNumber, Vec<crate::types::PullRequestFile>)>>>
    {
        // Fetch file stats from all repositories concurrently
        let fetch_futures = pr_numbers_of_repositories
            .into_iter()
            .map(|(repo_id, pr_numbers)| {
                let github_client = self.github_client.clone();

                async move {
                    let mut repo_files = Vec::new();

                    // Fetch each PR file stats sequentially to avoid overwhelming the API
                    for pr_number in pr_numbers {
                        match github_client
                            .fetch_pull_request_files(repo_id.clone(), pr_number)
                            .await
                        {
                            Ok(files) => {
                                repo_files.push((pr_number, files));
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "Failed to fetch file stats for PR #{} from {}: {}",
                                    pr_number.value(),
                                    repo_id,
                                    e
                                );
                                // Continue to next PR instead of failing completely
                            }
                        }
                    }

                    Ok::<_, anyhow::Error>((repo_id, repo_files))
                }
            });

        let results: Vec<
            Result<(
                RepositoryId,
                Vec<(PullRequestNumber, Vec<crate::types::PullRequestFile>)>,
            )>,
        > = stream::iter(fetch_futures)
            .buffer_unordered(10) // Process up to 10 repositories concurrently
            .collect()
            .await;

        let files_by_repo: BTreeMap<
            RepositoryId,
            Vec<(PullRequestNumber, Vec<crate::types::PullRequestFile>)>,
        > = results
            .into_iter()
            .filter_map(|result| result.ok())
            .collect();

        Ok(files_by_repo)
    }
}
