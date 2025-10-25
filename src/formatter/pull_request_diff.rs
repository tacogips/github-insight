use crate::types::{PullRequestNumber, RepositoryId};

use super::MarkdownContent;

/// Format a pull request diff into markdown
///
/// This function formats a unified diff for a pull request into markdown format,
/// displaying the repository, PR number, and the diff in a code block.
///
/// # Arguments
///
/// * `repository_id` - The repository identifier
/// * `pr_number` - The pull request number
/// * `diff` - The unified diff content
///
/// # Returns
///
/// Returns a `MarkdownContent` containing the formatted diff
pub fn pull_request_diff_markdown(
    repository_id: &RepositoryId,
    pr_number: PullRequestNumber,
    diff: &str,
) -> MarkdownContent {
    let mut content = String::new();

    // Header with repository and PR number
    content.push_str(&format!(
        "## Pull Request: {}/pull/{}\n\n",
        repository_id.full_name(),
        pr_number.value()
    ));

    // Diff in code block
    content.push_str("```diff\n");
    content.push_str(diff);
    if !diff.ends_with('\n') {
        content.push('\n');
    }
    content.push_str("```\n");

    MarkdownContent(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pull_request_diff_markdown() {
        let repo_id = RepositoryId::new("owner".to_string(), "repo".to_string());
        let pr_number = PullRequestNumber::new(123);
        let diff = "diff --git a/file.txt b/file.txt\nindex 1234567..abcdefg 100644\n--- a/file.txt\n+++ b/file.txt\n@@ -1,3 +1,4 @@\n line1\n-line2\n+line2 modified\n+line3 added\n line4";

        let result = pull_request_diff_markdown(&repo_id, pr_number, diff);

        assert!(result.0.contains("## Pull Request: owner/repo/pull/123"));
        assert!(result.0.contains("```diff\n"));
        assert!(result.0.contains(diff));
        assert!(result.0.ends_with("```\n"));
    }

    #[test]
    fn test_pull_request_diff_markdown_with_trailing_newline() {
        let repo_id = RepositoryId::new("owner".to_string(), "repo".to_string());
        let pr_number = PullRequestNumber::new(456);
        let diff = "diff --git a/test.rs b/test.rs\n";

        let result = pull_request_diff_markdown(&repo_id, pr_number, diff);

        assert!(result.0.contains("## Pull Request: owner/repo/pull/456"));
        // Should not have double newlines before closing code block
        assert!(!result.0.ends_with("\n\n```\n"));
        assert!(result.0.ends_with("\n```\n"));
    }
}
