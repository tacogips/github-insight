use crate::{tools::GitInsightTools, types::ProfileName};
use anyhow::Result;
use rmcp::transport::sse_server::SseServer;
use std::net::SocketAddr;

pub struct SseServerApp {
    bind_addr: SocketAddr,
    github_token: Option<String>,
    timezone: Option<String>,
    profile_name: Option<ProfileName>,
}

impl SseServerApp {
    /// Creates a new SSE server application instance.
    ///
    /// # Arguments
    ///
    /// * `bind_addr` - The socket address to bind the server to
    /// * `github_token` - Optional GitHub personal access token for API authentication
    ///
    /// # Returns
    ///
    /// Returns a new SseServerApp instance.
    pub fn new(
        bind_addr: SocketAddr,
        github_token: Option<String>,
        timezone: Option<String>,
        profile_name: Option<ProfileName>,
    ) -> Self {
        Self {
            bind_addr,
            github_token,
            timezone,
            profile_name,
        }
    }

    /// Starts the SSE server and serves GitInsightTools over Server-Sent Events.
    ///
    /// This method starts the server and waits for a Ctrl+C signal to shutdown gracefully.
    ///
    /// # Returns
    ///
    /// Returns Ok(()) when the server shuts down gracefully.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The server fails to bind to the specified address
    /// - The server encounters an error during operation
    pub async fn serve(self) -> Result<()> {
        // Initialize the service before starting the server
        // This ensures the database is set up and performs initial sync
        tracing::info!("Initializing GitInsight service before starting SSE server...");
        let init_service = GitInsightTools::new(
            self.github_token.clone(),
            self.timezone.clone(),
            self.profile_name.clone(),
        );
        init_service.initialize().await?;
        tracing::info!("GitInsight service initialization complete");

        let sse_server = SseServer::serve(self.bind_addr).await?;
        let github_token = self.github_token.clone();
        let timezone = self.timezone.clone();
        let profile_name = self.profile_name.clone();
        let cancellation_token = sse_server.with_service(move || {
            GitInsightTools::new(github_token.clone(), timezone.clone(), profile_name.clone())
        });

        // Wait for Ctrl+C signal to gracefully shutdown
        tokio::signal::ctrl_c().await?;

        // Cancel the server
        cancellation_token.cancel();

        Ok(())
    }
}
