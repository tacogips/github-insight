use anyhow::Result;
use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt};

use github_insight::formatter::TimezoneOffset;
use github_insight::types::ProfileName;

/// Parse timezone if provided, otherwise use local timezone
fn parse_timezone_or_default(timezone: Option<String>) -> Option<String> {
    timezone
        .and_then(|tz| TimezoneOffset::parse(&tz).map(|_| tz))
        .or_else(|| Some(TimezoneOffset::from_local().to_string()))
}

#[derive(Parser)]
#[command(author, version = env!("CARGO_PKG_VERSION"))]
#[command(
    about = "GitHub Insight MCP Server - Model Context Protocol server for comprehensive GitHub data analysis and management"
)]
#[command(
    long_about = "GitHub Insight MCP Server provides comprehensive access to GitHub repositories, issues, pull requests, and projects through the Model Context Protocol. Features include multi-repository search with GitHub query syntax support, detailed issue and pull request fetching with comments and metadata, advanced project resource management with pagination support, and flexible timezone customization. Supports both stdio and HTTP/SSE interfaces for seamless integration with MCP clients like Claude Desktop."
)]
#[command(propagate_version = true)]
#[command(disable_version_flag = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the server in stdin/stdout mode for MCP client integration like Claude Desktop
    Stdio {
        /// Enable debug logging for troubleshooting and development
        #[arg(short, long)]
        debug: bool,

        /// GitHub personal access token for API authentication (overrides GITHUB_INSIGHT_GITHUB_TOKEN environment variable)
        #[arg(short = 't', long)]
        github_token: Option<String>,

        /// Timezone for datetime formatting in output - supports standard timezones (e.g., "JST", "+09:00", "America/New_York", "UTC")
        #[arg(short = 'z', long)]
        timezone: Option<String>,

        /// Profile name for database isolation and configuration management (default: "default")
        #[arg(short = 'p', long)]
        profile: Option<String>,
    },
    /// Run the server with HTTP/SSE interface for web-based access and testing
    Http {
        /// Address to bind the HTTP server to for web interface access
        #[arg(short, long, default_value = "0.0.0.0:8080")]
        address: String,

        /// Enable debug logging for troubleshooting and development
        #[arg(short, long)]
        debug: bool,

        /// GitHub personal access token for API authentication (overrides GITHUB_INSIGHT_GITHUB_TOKEN environment variable)
        #[arg(short = 't', long)]
        github_token: Option<String>,

        /// Timezone for datetime formatting in output - supports standard timezones (e.g., "JST", "+09:00", "America/New_York", "UTC")
        #[arg(short = 'z', long)]
        timezone: Option<String>,

        /// Profile name for database isolation and configuration management (default: "default")
        #[arg(short = 'p', long)]
        profile: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize rustls crypto provider early to prevent "no process-level CryptoProvider available" panics
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let cli = Cli::parse();

    match cli.command {
        Commands::Stdio {
            debug: _,
            github_token,
            timezone,
            profile,
        } => {
            // Use github_token directly or get from environment
            let github_token =
                github_token.or_else(|| std::env::var("GITHUB_INSIGHT_GITHUB_TOKEN").ok());

            // Parse timezone if provided, otherwise use local timezone
            let timezone = parse_timezone_or_default(timezone);

            github_insight::transport::stdio::run_stdio_server(
                github_token,
                timezone,
                profile.map(|p| ProfileName::from(p.as_str())),
            )
            .await
        }
        Commands::Http {
            address,
            debug,
            github_token,
            timezone,
            profile,
        } => {
            // Use github_token directly or get from environment
            let github_token =
                github_token.or_else(|| std::env::var("GITHUB_INSIGHT_GITHUB_TOKEN").ok());

            // Parse timezone if provided, otherwise use local timezone
            let timezone = parse_timezone_or_default(timezone);

            run_http_server(address, debug, github_token, timezone, profile).await
        }
    }
}

async fn run_http_server(
    address: String,
    debug: bool,
    github_token: Option<String>,
    timezone: Option<String>,
    profile_name: Option<String>,
) -> Result<()> {
    // Setup tracing
    let level = if debug { "debug" } else { "info" };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{},{}", level, env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer().with_ansi(false)) // Disable ANSI color codes
        .init();

    // Parse socket address
    let addr: SocketAddr = address.parse()?;

    tracing::debug!("Rust Documentation Server listening on {}", addr);
    tracing::info!(
        "Access the Rust Documentation Server at http://{}/sse",
        addr
    );

    if github_token.is_some() {
        tracing::info!("Using GitHub token from command line arguments");
    }

    // Create app and run server using the new rust-sdk implementation
    let app = github_insight::transport::sse_server::SseServerApp::new(
        addr,
        github_token,
        timezone,
        profile_name.map(|p| ProfileName::from(p.as_str())),
    );
    app.serve().await?;

    Ok(())
}
