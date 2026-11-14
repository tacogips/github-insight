//! Git repository operations for detecting current branch and repository information
//!
//! This module provides utilities for working with local Git repositories,
//! including detecting the current branch and repository URL.

use anyhow::{Context, Result, anyhow};
use git2::Repository;
use std::path::Path;

/// Represents information about the current Git repository
#[derive(Debug, Clone)]
pub struct GitRepositoryInfo {
    /// The current branch name
    pub current_branch: String,
    /// The repository URL (remote origin)
    pub repository_url: String,
}

/// Get the current branch name from a Git repository
///
/// # Arguments
///
/// * `repo_path` - Path to the Git repository (typically the working directory)
///
/// # Returns
///
/// * `Result<String>` - The current branch name or error
pub fn get_current_branch<P: AsRef<Path>>(repo_path: P) -> Result<String> {
    let repo = Repository::open(repo_path)
        .context("Failed to open git repository. Are you in a git repository?")?;

    let head = repo.head().context("Failed to get HEAD reference")?;

    if !head.is_branch() {
        return Err(anyhow!(
            "HEAD is not pointing to a branch (detached HEAD state)"
        ));
    }

    let branch_name = head
        .shorthand()
        .ok_or_else(|| anyhow!("Failed to get branch name from HEAD"))?
        .to_string();

    Ok(branch_name)
}

/// Get the remote URL for the origin remote
///
/// # Arguments
///
/// * `repo_path` - Path to the Git repository
///
/// # Returns
///
/// * `Result<String>` - The origin remote URL or error
pub fn get_remote_url<P: AsRef<Path>>(repo_path: P) -> Result<String> {
    let repo = Repository::open(repo_path).context("Failed to open git repository")?;

    let remote = repo
        .find_remote("origin")
        .context("Failed to find 'origin' remote")?;

    let url = remote
        .url()
        .ok_or_else(|| anyhow!("Remote 'origin' has no URL"))?
        .to_string();

    Ok(normalize_github_url(&url))
}

/// Get both current branch and repository URL
///
/// # Arguments
///
/// * `repo_path` - Path to the Git repository
///
/// # Returns
///
/// * `Result<GitRepositoryInfo>` - Repository information or error
pub fn get_repository_info<P: AsRef<Path>>(repo_path: P) -> Result<GitRepositoryInfo> {
    let path = repo_path.as_ref();
    let current_branch = get_current_branch(path)?;
    let repository_url = get_remote_url(path)?;

    Ok(GitRepositoryInfo {
        current_branch,
        repository_url,
    })
}

/// Normalize GitHub URL to standard HTTPS format
///
/// Converts various GitHub URL formats to the standard HTTPS format:
/// - SSH: git@github.com:owner/repo.git -> https://github.com/owner/repo
/// - HTTPS with .git: https://github.com/owner/repo.git -> https://github.com/owner/repo
///
/// # Arguments
///
/// * `url` - The URL to normalize
///
/// # Returns
///
/// * `String` - Normalized URL
fn normalize_github_url(url: &str) -> String {
    // Remove trailing .git if present
    let url = url.trim_end_matches(".git");

    // Convert SSH format to HTTPS
    if url.starts_with("git@github.com:") {
        let repo_path = url.strip_prefix("git@github.com:").unwrap();
        return format!("https://github.com/{}", repo_path);
    }

    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_github_url_ssh() {
        let url = "git@github.com:tacogips/github-insight.git";
        let normalized = normalize_github_url(url);
        assert_eq!(normalized, "https://github.com/tacogips/github-insight");
    }

    #[test]
    fn test_normalize_github_url_https() {
        let url = "https://github.com/tacogips/github-insight.git";
        let normalized = normalize_github_url(url);
        assert_eq!(normalized, "https://github.com/tacogips/github-insight");
    }

    #[test]
    fn test_normalize_github_url_https_no_git() {
        let url = "https://github.com/tacogips/github-insight";
        let normalized = normalize_github_url(url);
        assert_eq!(normalized, "https://github.com/tacogips/github-insight");
    }
}
