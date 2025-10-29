use crate::types::{PullRequestFile, PullRequestNumber, RepositoryId};

use super::MarkdownContent;

/// Format pull request file statistics into markdown
///
/// This function formats file statistics (changed files list with additions, deletions,
/// and change counts) for a pull request into markdown format.
///
/// # Arguments
///
/// * `repository_id` - The repository identifier
/// * `pr_number` - The pull request number
/// * `files` - Vector of file metadata including statistics
///
/// # Returns
///
/// Returns a `MarkdownContent` containing the formatted file statistics
pub fn pull_request_file_stats_markdown(
    repository_id: &RepositoryId,
    pr_number: PullRequestNumber,
    files: &[PullRequestFile],
) -> MarkdownContent {
    let mut content = String::new();

    // Header with repository and PR number
    content.push_str(&format!(
        "## Pull Request Files: {}/pull/{}\n\n",
        repository_id.full_name(),
        pr_number.value()
    ));

    if files.is_empty() {
        content.push_str("No files changed.\n");
        return MarkdownContent(content);
    }

    // Summary statistics
    let total_additions: u32 = files.iter().map(|f| f.additions).sum();
    let total_deletions: u32 = files.iter().map(|f| f.deletions).sum();
    let total_changes: u32 = files.iter().map(|f| f.changes).sum();
    let file_count = files.len();

    content.push_str(&format!(
        "**Summary:** {} file(s) changed, +{} additions, -{} deletions, {} total changes\n\n",
        file_count, total_additions, total_deletions, total_changes
    ));

    // File list table
    content.push_str("| File | Status | Additions | Deletions | Changes |\n");
    content.push_str("|------|--------|-----------|-----------|----------|\n");

    for file in files {
        let filename = if let Some(prev) = &file.previous_filename {
            format!("{} → {}", prev, file.filename)
        } else {
            file.filename.clone()
        };

        content.push_str(&format!(
            "| {} | {} | +{} | -{} | {} |\n",
            filename, file.status, file.additions, file.deletions, file.changes
        ));
    }

    content.push('\n');

    MarkdownContent(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_file(
        filename: &str,
        status: &str,
        additions: u32,
        deletions: u32,
    ) -> PullRequestFile {
        PullRequestFile {
            sha: "abc123".to_string(),
            filename: filename.to_string(),
            status: status.to_string(),
            additions,
            deletions,
            changes: additions + deletions,
            blob_url: format!("https://github.com/owner/repo/blob/main/{}", filename),
            raw_url: format!("https://github.com/owner/repo/raw/main/{}", filename),
            contents_url: format!(
                "https://api.github.com/repos/owner/repo/contents/{}",
                filename
            ),
            patch: None,
            previous_filename: None,
        }
    }

    #[test]
    fn test_pull_request_file_stats_markdown() {
        let repo_id = RepositoryId::new("owner".to_string(), "repo".to_string());
        let pr_number = PullRequestNumber::new(123);
        let files = vec![
            create_test_file("src/main.rs", "modified", 10, 5),
            create_test_file("README.md", "modified", 3, 1),
            create_test_file("src/lib.rs", "added", 50, 0),
        ];

        let result = pull_request_file_stats_markdown(&repo_id, pr_number, &files);

        assert!(
            result
                .0
                .contains("## Pull Request Files: owner/repo/pull/123")
        );
        assert!(result.0.contains("3 file(s) changed"));
        assert!(result.0.contains("+63 additions"));
        assert!(result.0.contains("-6 deletions"));
        assert!(result.0.contains("69 total changes"));
        assert!(result.0.contains("src/main.rs"));
        assert!(result.0.contains("README.md"));
        assert!(result.0.contains("src/lib.rs"));
        assert!(
            result
                .0
                .contains("| File | Status | Additions | Deletions | Changes |")
        );
    }

    #[test]
    fn test_pull_request_file_stats_markdown_empty() {
        let repo_id = RepositoryId::new("owner".to_string(), "repo".to_string());
        let pr_number = PullRequestNumber::new(456);
        let files = vec![];

        let result = pull_request_file_stats_markdown(&repo_id, pr_number, &files);

        assert!(
            result
                .0
                .contains("## Pull Request Files: owner/repo/pull/456")
        );
        assert!(result.0.contains("No files changed."));
    }

    #[test]
    fn test_pull_request_file_stats_markdown_with_rename() {
        let repo_id = RepositoryId::new("owner".to_string(), "repo".to_string());
        let pr_number = PullRequestNumber::new(789);
        let mut file = create_test_file("src/new_name.rs", "renamed", 0, 0);
        file.previous_filename = Some("src/old_name.rs".to_string());
        let files = vec![file];

        let result = pull_request_file_stats_markdown(&repo_id, pr_number, &files);

        assert!(result.0.contains("src/old_name.rs → src/new_name.rs"));
        assert!(result.0.contains("renamed"));
    }
}
