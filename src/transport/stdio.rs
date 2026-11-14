use crate::tools::GitInsightTools;
use crate::types::ProfileName;
use anyhow::Result;
use rmcp::ServiceExt;
use rmcp::transport::stdio;

/// Runs the MCP server in STDIN/STDOUT mode.
///
/// This mode is used when the server is launched as a subprocess by an MCP client,
/// communicating through standard input/output streams.
///
/// # Arguments
/// * `github_token` - Optional GitHub personal access token for API authentication
/// * `repository_cache_dir` - Optional custom directory for caching repository data
/// * `timezone` - Optional timezone for displaying dates
/// * `profile_name` - Optional profile name for database isolation
///
/// # Returns
/// * `Result<()>` - Success when server shuts down cleanly, or error
///
/// # Example
/// ```no_run
/// # use github_insight::transport::stdio::run_stdio_server;
/// # async fn example() -> anyhow::Result<()> {
/// run_stdio_server(
///     Some("ghp_xxxxxxxxxxxx".to_string()),
///     None,
///     None
/// ).await?;
/// # Ok(())
/// # }
/// ```
pub async fn run_stdio_server(
    github_token: Option<String>,
    timezone: Option<String>,
    profile_name: Option<ProfileName>,
) -> Result<()> {
    // Create an instance of our GitHub code tools wrapper with the provided token and profile name
    let service = GitInsightTools::new(github_token, timezone, profile_name);

    // Initialize the service and perform initial sync
    service.initialize().await?;

    // Use the new rust-sdk stdio transport implementation
    let server = service.serve(stdio()).await?;

    server.waiting().await?;
    Ok(())
}
