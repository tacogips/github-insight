use crate::types::PullRequestUrl;

/// Markdown representation of pull request diff contents
#[derive(Debug, Clone)]
pub struct PullRequestDiffContentsMarkdown(pub String);

/// Format pull request diff contents as markdown with optional skip/limit information
///
/// # Arguments
///
/// * `pull_request_url` - Pull request URL
/// * `file_path` - File path within the repository
/// * `diff_content` - Unified diff content
/// * `skip` - Optional number of lines skipped from the beginning
/// * `limit` - Optional maximum number of lines returned
///
/// # Returns
///
/// Returns formatted markdown with diff content in a code block
pub fn pull_request_diff_contents_markdown(
    pull_request_url: &PullRequestUrl,
    file_path: &str,
    diff_content: &str,
    skip: Option<u32>,
    limit: Option<u32>,
) -> PullRequestDiffContentsMarkdown {
    let mut output = String::new();

    // Header
    output.push_str(&format!("## Diff for file: {}\n", file_path));
    output.push_str(&format!("**Pull Request:** {}\n", pull_request_url.0));

    // Skip/limit information if present
    if let Some(skip_val) = skip {
        output.push_str(&format!("**Skip:** {} lines\n", skip_val));
    }
    if let Some(limit_val) = limit {
        output.push_str(&format!("**Limit:** {} lines\n", limit_val));
    }

    // Diff content
    output.push('\n');
    output.push_str("```diff\n");
    output.push_str(diff_content);
    output.push_str("\n```\n");

    PullRequestDiffContentsMarkdown(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pull_request_diff_contents_markdown_full() {
        let pr_url = PullRequestUrl("https://github.com/owner/repo/pull/123".to_string());
        let file_path = "src/main.rs";
        let diff_content = "@@ -1,3 +1,3 @@\n fn main() {\n-    println!(\"Hello\");\n+    println!(\"World\");\n }";

        let result =
            pull_request_diff_contents_markdown(&pr_url, file_path, diff_content, None, None);

        assert!(result.0.contains("## Diff for file: src/main.rs"));
        assert!(
            result
                .0
                .contains("**Pull Request:** https://github.com/owner/repo/pull/123")
        );
        assert!(result.0.contains("```diff"));
        assert!(result.0.contains(diff_content));
        assert!(!result.0.contains("**Skip:**"));
        assert!(!result.0.contains("**Limit:**"));
    }

    #[test]
    fn test_pull_request_diff_contents_markdown_with_skip_limit() {
        let pr_url = PullRequestUrl("https://github.com/owner/repo/pull/456".to_string());
        let file_path = "README.md";
        let diff_content = "@@ -10,5 +10,5 @@\n Some content";

        let result = pull_request_diff_contents_markdown(
            &pr_url,
            file_path,
            diff_content,
            Some(10),
            Some(20),
        );

        assert!(result.0.contains("## Diff for file: README.md"));
        assert!(result.0.contains("**Skip:** 10 lines"));
        assert!(result.0.contains("**Limit:** 20 lines"));
        assert!(result.0.contains("```diff"));
    }

    #[test]
    fn test_pull_request_diff_contents_markdown_with_skip_only() {
        let pr_url = PullRequestUrl("https://github.com/owner/repo/pull/789".to_string());
        let file_path = "lib.rs";
        let diff_content = "diff content";

        let result =
            pull_request_diff_contents_markdown(&pr_url, file_path, diff_content, Some(5), None);

        assert!(result.0.contains("**Skip:** 5 lines"));
        assert!(!result.0.contains("**Limit:**"));
    }

    #[test]
    fn test_pull_request_diff_contents_markdown_with_limit_only() {
        let pr_url = PullRequestUrl("https://github.com/owner/repo/pull/321".to_string());
        let file_path = "test.rs";
        let diff_content = "diff content";

        let result =
            pull_request_diff_contents_markdown(&pr_url, file_path, diff_content, None, Some(15));

        assert!(!result.0.contains("**Skip:**"));
        assert!(result.0.contains("**Limit:** 15 lines"));
    }
}
