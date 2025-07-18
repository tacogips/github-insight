use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use std::env;
use std::time::Duration;
use tracing_subscriber::EnvFilter;

use github_insight::formatter::{
    TimezoneOffset, issue_body_markdown_with_timezone, issue_body_markdown_with_timezone_light,
    project_body_markdown_with_timezone, project_resource_body_markdown_with_timezone,
    project_resource_body_markdown_with_timezone_light, pull_request_body_markdown_with_timezone,
    pull_request_body_markdown_with_timezone_light, repository_body_markdown_with_timezone,
};

/// Parse timezone if provided, otherwise use local timezone
fn parse_timezone_or_default(timezone: Option<String>) -> Option<TimezoneOffset> {
    timezone
        .and_then(|tz| TimezoneOffset::parse(&tz))
        .or_else(|| Some(TimezoneOffset::from_local()))
}
use github_insight::github::GitHubClient;
use github_insight::services::{ProfileService, default_profile_config_dir};
use github_insight::tools::functions;
use github_insight::types::project::{ProjectNumber, ProjectUrl};
use github_insight::types::repository::{Owner, RepositoryName};
use github_insight::types::{
    IssueUrl, OutputOption, ProfileName, ProjectId, PullRequestUrl, RepositoryId, RepositoryUrl,
    SearchQuery,
};

#[derive(Parser)]
#[command(name = "github-insight-cli")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(
    about = "GitHub Insight CLI - Comprehensive GitHub repository and project management tool with advanced search, issue tracking, and project resource management capabilities"
)]
#[command(
    long_about = "GitHub Insight CLI provides comprehensive tools for managing GitHub repositories and projects through configurable profiles. Features include multi-repository search with GitHub query syntax support, detailed issue and pull request fetching with comments and metadata, advanced project resource management with pagination support, and flexible output formatting (JSON/Markdown) with timezone customization. Perfect for developers, project managers, and teams who need efficient GitHub workflow management."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// Output format for results - markdown provides formatted display, json for programmatic use and API integration
    #[arg(long, global = true, default_value = "markdown")]
    format: OutputFormat,
    /// GitHub personal access token for API access (can also be set via GITHUB_TOKEN or GITHUB_INSIGHT_GITHUB_TOKEN environment variables)
    #[arg(long, global = true)]
    github_token: Option<String>,
    /// Timezone for datetime formatting in output - supports standard timezones (e.g., "JST", "+09:00", "America/New_York", "UTC")
    #[arg(long, global = true)]
    timezone: Option<String>,
    /// Request timeout in seconds for GitHub API calls - useful for slow networks or large data sets (default: 30 seconds)
    #[arg(long, global = true)]
    request_timeout: Option<u64>,
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Json,
    Markdown,
}

#[derive(Clone, ValueEnum)]
enum OutputOptionCli {
    Light,
    Rich,
}

impl From<OutputOptionCli> for OutputOption {
    fn from(cli_option: OutputOptionCli) -> Self {
        match cli_option {
            OutputOptionCli::Light => OutputOption::Light,
            OutputOptionCli::Rich => OutputOption::Rich,
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Register a repository to a profile for centralized management and search operations across multiple repositories
    RegisterRepo {
        /// Repository URL in GitHub format (e.g., <https://github.com/owner/repo>) - supports both .git and non-.git URLs
        repository_url: String,
        /// Profile name for organizing repositories (default: "default")
        #[arg(short, long, default_value = "default")]
        profile: String,
    },
    /// Remove a repository from a profile, excluding it from search and management operations
    UnregisterRepo {
        /// Repository URL to remove from profile
        repository_url: String,
        /// Profile name containing the repository (default: "default")
        #[arg(short, long, default_value = "default")]
        profile: String,
    },
    /// Register a GitHub project to a profile for comprehensive resource management and tracking with pagination support
    RegisterProject {
        /// GitHub project URL - supports both user and organization projects (e.g., <https://github.com/users/username/projects/1> or <https://github.com/orgs/orgname/projects/1>)
        project_url: String,
        /// Profile name for organizing projects (default: "default")
        #[arg(short, long, default_value = "default")]
        profile: String,
    },
    /// Remove a GitHub project from a profile, excluding it from resource management and tracking
    UnregisterProject {
        /// Project URL to remove from profile
        project_url: String,
        /// Profile name containing the project (default: "default")
        #[arg(short, long, default_value = "default")]
        profile: String,
    },
    /// Display all available profiles with their configurations and metadata
    ListProfiles,
    /// Display all repositories registered in a specific profile with their URLs and registration details
    ListRepos {
        /// Profile name to list repositories from (default: "default")
        #[arg(short, long, default_value = "default")]
        profile: String,
    },
    /// Display all GitHub projects registered in a specific profile with their URLs and metadata
    ListProjects {
        /// Profile name to list projects from (default: "default")
        #[arg(short, long, default_value = "default")]
        profile: String,
    },
    /// Create a new profile for organizing repositories and projects with optional description
    CreateProfile {
        /// Profile name - must be unique and contain only alphanumeric characters, hyphens, and underscores
        name: String,
        /// Optional profile description for better organization and documentation
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Delete a profile and all its associated repository and project registrations (irreversible)
    DeleteProfile {
        /// Profile name to delete permanently
        name: String,
    },
    /// Search for issues and pull requests across multiple repositories with advanced GitHub search syntax and pagination support
    Search {
        /// Search query text - supports full GitHub search syntax (e.g., "is:issue state:open author:username", "is:pr label:bug", "created:>2024-01-01"). Note: Repository specifications (repo:owner/name) are not supported in the query and will be ignored - use the --repository option or register repositories in the profile instead
        query: String,
        /// Profile name containing repositories to search (default: "default")
        #[arg(short, long, default_value = "default")]
        profile: String,
        /// Optional repository to limit search scope - format: GitHub URL (e.g., "https://github.com/microsoft/vscode")
        #[arg(short, long)]
        repository_url: Option<String>,
        /// Maximum number of results to return - useful for controlling output size (default: 30, max: 100)
        #[arg(short, long, default_value = "30")]
        limit: usize,
        /// Output format for search results - light provides minimal information, rich provides comprehensive details (default: light)
        #[arg(long, default_value = "light")]
        output: OutputOptionCli,
    },
    /// Fetch detailed project resources including items, metadata, timestamps, and assignees with comprehensive pagination support
    GetProjectResources {
        /// Optional project URL to fetch resources from - if not provided, fetches all projects from profile for batch processing
        project_url: Option<String>,
        /// Profile name containing projects to fetch resources from (default: "default")
        #[arg(short, long, default_value = "default")]
        profile: String,
        /// Output format for project resources - light provides minimal information, rich provides comprehensive details (default: rich)
        #[arg(long, default_value = "rich")]
        output: OutputOptionCli,
    },
    /// Fetch detailed issue information including comments, metadata, labels, and timeline events by URLs
    GetIssues {
        /// GitHub issue URLs to fetch detailed information from - supports multiple URLs for batch processing
        urls: Vec<String>,
    },
    /// Fetch detailed pull request information including comments, metadata, reviews, and timeline events by URLs
    GetPullRequests {
        /// GitHub pull request URLs to fetch detailed information from - supports multiple URLs for batch processing
        urls: Vec<String>,
    },
    /// Fetch detailed repository information including metadata, statistics, releases (with configurable limit), and configuration by URLs
    GetRepositories {
        /// GitHub repository URLs to fetch detailed information from - supports multiple URLs for batch processing
        urls: Vec<String>,
        /// Optional limit for number of releases to show per repository (default: 10)
        #[arg(long)]
        showing_release_limit: Option<usize>,
        /// Optional limit for number of milestones to show per repository (default: 10)
        #[arg(long)]
        showing_milestone_limit: Option<usize>,
    },
    /// Fetch detailed project information including metadata, description, and timestamps by URLs
    GetProjects {
        /// GitHub project URLs to fetch detailed information from - supports multiple URLs for batch processing
        urls: Vec<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize rustls crypto provider early to prevent "no process-level CryptoProvider available" panics
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("github-insight=info".parse()?),
        )
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();

    // Get GitHub token from CLI or environment
    let github_token = cli
        .github_token
        .or_else(|| env::var("GITHUB_INSIGHT_GITHUB_TOKEN").ok());

    // Parse timezone if provided, otherwise use local timezone
    let timezone = parse_timezone_or_default(cli.timezone);

    // Initialize profile service
    let config_dir = default_profile_config_dir()
        .map_err(|e| anyhow::anyhow!("Failed to get config directory: {}", e))?;

    let mut profile_service = ProfileService::new(config_dir)
        .map_err(|e| anyhow::anyhow!("Failed to initialize profile service: {}", e))?;

    match cli.command {
        Commands::RegisterRepo {
            repository_url,
            profile,
        } => {
            let repo_id = parse_repository_url(&repository_url)?;
            profile_service
                .register_repository(&ProfileName::from(profile.as_str()), repo_id)
                .map_err(|e| anyhow::anyhow!("Failed to register repository: {}", e))?;
            println!(
                "Successfully registered repository '{}' to profile '{}'",
                repository_url, profile
            );
        }
        Commands::UnregisterRepo {
            repository_url,
            profile,
        } => {
            let repo_id = parse_repository_url(&repository_url)?;
            profile_service
                .unregister_repository(&ProfileName::from(profile.as_str()), &repo_id)
                .map_err(|e| anyhow::anyhow!("Failed to unregister repository: {}", e))?;
            println!(
                "Successfully unregistered repository '{}' from profile '{}'",
                repository_url, profile
            );
        }
        Commands::RegisterProject {
            project_url,
            profile,
        } => {
            let project_id = parse_project_url(&project_url)?;
            profile_service
                .register_project(&ProfileName::from(profile.as_str()), project_id)
                .map_err(|e| anyhow::anyhow!("Failed to register project: {}", e))?;
            println!(
                "Successfully registered project '{}' to profile '{}'",
                project_url, profile
            );
        }
        Commands::UnregisterProject {
            project_url,
            profile,
        } => {
            let project_id = parse_project_url(&project_url)?;
            profile_service
                .unregister_project(&ProfileName::from(profile.as_str()), &project_id)
                .map_err(|e| anyhow::anyhow!("Failed to unregister project: {}", e))?;
            println!(
                "Successfully unregistered project '{}' from profile '{}'",
                project_url, profile
            );
        }
        Commands::ListProfiles => {
            let profiles = profile_service.list_profiles();
            if profiles.is_empty() {
                println!("No profiles found");
            } else {
                println!("Profiles:");
                for profile in profiles {
                    println!("  - {}", profile);
                }
            }
        }
        Commands::ListRepos { profile } => {
            let repos = profile_service
                .list_repositories(&ProfileName::from(profile.as_str()))
                .map_err(|e| anyhow::anyhow!("Failed to list repositories: {}", e))?;
            if repos.is_empty() {
                println!("No repositories found in profile '{}'", profile);
            } else {
                println!("Repositories in profile '{}':", profile);
                for repo in repos {
                    println!("  - {}", repo);
                }
            }
        }
        Commands::ListProjects { profile } => {
            let projects = profile_service
                .list_projects(&ProfileName::from(profile.as_str()))
                .map_err(|e| anyhow::anyhow!("Failed to list projects: {}", e))?;
            if projects.is_empty() {
                println!("No projects found in profile '{}'", profile);
            } else {
                println!("Projects in profile '{}':", profile);
                for project in projects {
                    println!("  - {}", project);
                }
            }
        }
        Commands::CreateProfile { name, description } => {
            profile_service
                .create_profile(&ProfileName::from(name.as_str()), description)
                .map_err(|e| anyhow::anyhow!("Failed to create profile: {}", e))?;
            println!("Successfully created profile '{}'", name);
        }
        Commands::DeleteProfile { name } => {
            profile_service
                .delete_profile(&ProfileName::from(name.as_str()))
                .map_err(|e| anyhow::anyhow!("Failed to delete profile: {}", e))?;
            println!("Successfully deleted profile '{}'", name);
        }
        Commands::Search {
            query,
            profile,
            repository_url,
            limit,
            output,
        } => {
            handle_search_command(SearchParams {
                query: &query,
                profile: &profile,
                repository_url: &repository_url,
                limit,
                format: &cli.format,
                output_option: &output.into(),
                github_token: &github_token,
                timezone: &timezone,
            })
            .await?;
        }
        Commands::GetProjectResources {
            project_url,
            profile,
            output,
        } => {
            handle_get_project_resources_command(
                &project_url,
                &profile,
                &cli.format,
                &output.into(),
                &github_token,
                &timezone,
                &mut profile_service,
            )
            .await?;
        }
        Commands::GetIssues { urls } => {
            let issue_urls: Vec<IssueUrl> = urls.iter().map(|url| IssueUrl(url.clone())).collect();
            handle_get_issues_command(
                issue_urls,
                &cli.format,
                &github_token,
                &timezone,
                cli.request_timeout.map(Duration::from_secs),
            )
            .await?;
        }
        Commands::GetPullRequests { urls } => {
            let pull_request_urls: Vec<PullRequestUrl> =
                urls.iter().map(|url| PullRequestUrl(url.clone())).collect();
            handle_get_pull_requests_command(
                pull_request_urls,
                &cli.format,
                &github_token,
                &timezone,
                cli.request_timeout.map(Duration::from_secs),
            )
            .await?;
        }
        Commands::GetRepositories {
            urls,
            showing_release_limit,
            showing_milestone_limit,
        } => {
            let repository_urls: Vec<RepositoryUrl> =
                urls.iter().map(|url| RepositoryUrl(url.clone())).collect();
            handle_get_repositories_command(
                repository_urls,
                &cli.format,
                &github_token,
                &timezone,
                cli.request_timeout.map(Duration::from_secs),
                showing_release_limit,
                showing_milestone_limit,
            )
            .await?;
        }
        Commands::GetProjects { urls } => {
            let project_urls: Vec<ProjectUrl> =
                urls.iter().map(|url| ProjectUrl(url.clone())).collect();
            handle_get_projects_command(
                project_urls,
                &cli.format,
                &github_token,
                &timezone,
                cli.request_timeout.map(Duration::from_secs),
            )
            .await?;
        }
    }

    Ok(())
}

/// Parse repository URL into RepositoryId
fn parse_repository_url(url: &str) -> Result<RepositoryId> {
    // Simple URL parsing for GitHub URLs
    // This is a basic implementation - the actual parsing should be in the domain type
    if let Some(captures) = regex::Regex::new(r"https://github\.com/([^/]+)/([^/]+?)(?:\.git)?/?$")
        .unwrap()
        .captures(url)
    {
        let owner = captures.get(1).unwrap().as_str();
        let repo_name = captures.get(2).unwrap().as_str();

        Ok(RepositoryId {
            owner: Owner::from(owner),
            repository_name: RepositoryName::from(repo_name),
        })
    } else {
        Err(anyhow::anyhow!("Invalid repository URL format: {}", url))
    }
}

/// Parse project URL into ProjectId
fn parse_project_url(url: &str) -> Result<ProjectId> {
    let project_url = ProjectUrl(url.to_string());
    let (owner, number, project_type) = ProjectId::parse_url(&project_url)
        .map_err(|e| anyhow::anyhow!("Failed to parse project URL: {}", e))?;

    Ok(ProjectId::new(
        Owner::from(owner),
        ProjectNumber(number),
        project_type,
    ))
}

/// Search command parameters
struct SearchParams<'a> {
    query: &'a str,
    profile: &'a str,
    repository_url: &'a Option<String>,
    limit: usize,
    format: &'a OutputFormat,
    output_option: &'a OutputOption,
    github_token: &'a Option<String>,
    timezone: &'a Option<TimezoneOffset>,
}

/// Handle search command
async fn handle_search_command(params: SearchParams<'_>) -> Result<()> {
    let github_client = GitHubClient::new(params.github_token.clone(), None)
        .map_err(|e| anyhow::anyhow!("Failed to create GitHub client: {}", e))?;

    // Get profile service to load repositories
    let config_dir = default_profile_config_dir()
        .map_err(|e| anyhow::anyhow!("Failed to get config directory: {}", e))?;
    let profile_service = ProfileService::new(config_dir)
        .map_err(|e| anyhow::anyhow!("Failed to initialize profile service: {}", e))?;

    let repositories = if let Some(repo_str) = params.repository_url {
        // Parse single repository
        let repo_id = parse_repository_url(repo_str)?;
        vec![repo_id]
    } else {
        // Get all repositories from profile
        profile_service
            .list_repositories(&ProfileName::from(params.profile))
            .map_err(|e| anyhow::anyhow!("Failed to list repositories: {}", e))?
    };

    if repositories.is_empty() {
        println!("No repositories found. Please register repositories first.");
        return Ok(());
    }

    // Search for resources
    let search_query = SearchQuery::new(params.query.to_string());
    let search_result = functions::search::search_resources(
        &github_client,
        repositories,
        search_query,
        Some(params.limit as u32),
        None,
    )
    .await?;

    // Output results
    match params.format {
        OutputFormat::Json => {
            let json_output = serde_json::to_string_pretty(&search_result.results)?;
            println!("{}", json_output);
        }
        OutputFormat::Markdown => {
            if search_result.results.is_empty() {
                println!("No results found.");
            } else {
                for result in search_result.results {
                    let formatted = match result {
                        github_insight::types::IssueOrPullrequest::Issue(issue) => {
                            match params.output_option {
                                OutputOption::Light => {
                                    issue_body_markdown_with_timezone_light(
                                        &issue,
                                        params.timezone.as_ref(),
                                    )
                                    .0
                                }
                                OutputOption::Rich => {
                                    issue_body_markdown_with_timezone(
                                        &issue,
                                        params.timezone.as_ref(),
                                    )
                                    .0
                                }
                            }
                        }
                        github_insight::types::IssueOrPullrequest::PullRequest(pr) => {
                            match params.output_option {
                                OutputOption::Light => {
                                    pull_request_body_markdown_with_timezone_light(
                                        &pr,
                                        params.timezone.as_ref(),
                                    )
                                    .0
                                }
                                OutputOption::Rich => {
                                    pull_request_body_markdown_with_timezone(
                                        &pr,
                                        params.timezone.as_ref(),
                                    )
                                    .0
                                }
                            }
                        }
                    };
                    println!("{}", formatted);
                    println!("---");
                }
            }
        }
    }

    Ok(())
}

/// Handle get project resources command
async fn handle_get_project_resources_command(
    project_url: &Option<String>,
    profile: &str,
    format: &OutputFormat,
    output_option: &OutputOption,
    github_token: &Option<String>,
    timezone: &Option<TimezoneOffset>,
    profile_service: &mut ProfileService,
) -> Result<()> {
    let github_client = GitHubClient::new(github_token.clone(), None)
        .map_err(|e| anyhow::anyhow!("Failed to create GitHub client: {}", e))?;

    let project_resources = if let Some(project_url_str) = project_url {
        // Get resources for specific project
        let project_url = ProjectUrl(project_url_str.clone());
        functions::project::get_project_resources(&github_client, project_url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get project resources: {}", e))?
    } else {
        // Get resources for all projects in profile
        let project_ids = profile_service
            .list_projects(&ProfileName::from(profile))
            .map_err(|e| anyhow::anyhow!("Failed to list projects: {}", e))?;

        if project_ids.is_empty() {
            println!("No projects found in profile '{}'", profile);
            return Ok(());
        }

        functions::project::get_multiple_project_resources(&github_client, project_ids)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get project resources: {}", e))?
    };

    // Output results
    match format {
        OutputFormat::Json => {
            let json_output = serde_json::to_string_pretty(&project_resources)?;
            println!("{}", json_output);
        }
        OutputFormat::Markdown => {
            if project_resources.is_empty() {
                println!("No project resources found.");
            } else {
                for resource in project_resources {
                    let formatted = match output_option {
                        OutputOption::Light => project_resource_body_markdown_with_timezone_light(
                            &resource,
                            timezone.as_ref(),
                        ),
                        OutputOption::Rich => project_resource_body_markdown_with_timezone(
                            &resource,
                            timezone.as_ref(),
                        ),
                    };
                    println!("{}", formatted.0);
                    println!("---");
                }
            }
        }
    }

    Ok(())
}

/// Handle get issues command
async fn handle_get_issues_command(
    issue_urls: Vec<IssueUrl>,
    format: &OutputFormat,
    github_token: &Option<String>,
    timezone: &Option<TimezoneOffset>,
    request_timeout: Option<Duration>,
) -> Result<()> {
    let github_client = GitHubClient::new(github_token.clone(), request_timeout)
        .map_err(|e| anyhow::anyhow!("Failed to create GitHub client: {}", e))?;

    let issues_by_repo = functions::issue::get_issues_details(&github_client, issue_urls).await?;

    // Output results
    match format {
        OutputFormat::Json => {
            let json_output = serde_json::to_string_pretty(&issues_by_repo)?;
            println!("{}", json_output);
        }
        OutputFormat::Markdown => {
            let mut found_issues = false;
            for (_repo_id, issues) in issues_by_repo {
                for issue in issues {
                    let formatted = issue_body_markdown_with_timezone(&issue, timezone.as_ref());
                    println!("{}", formatted.0);
                    println!("---");
                    found_issues = true;
                }
            }
            if !found_issues {
                println!("No issues found for the provided URLs.");
            }
        }
    }

    Ok(())
}

/// Handle get pull requests command
async fn handle_get_pull_requests_command(
    pull_request_urls: Vec<PullRequestUrl>,
    format: &OutputFormat,
    github_token: &Option<String>,
    timezone: &Option<TimezoneOffset>,
    request_timeout: Option<Duration>,
) -> Result<()> {
    let github_client = GitHubClient::new(github_token.clone(), request_timeout)
        .map_err(|e| anyhow::anyhow!("Failed to create GitHub client: {}", e))?;

    let pull_requests_by_repo =
        functions::pull_request::get_pull_requests_details(&github_client, pull_request_urls)
            .await?;

    // Output results
    match format {
        OutputFormat::Json => {
            let json_output = serde_json::to_string_pretty(&pull_requests_by_repo)?;
            println!("{}", json_output);
        }
        OutputFormat::Markdown => {
            let mut found_prs = false;
            for (_repo_id, pull_requests) in pull_requests_by_repo {
                for pr in pull_requests {
                    let formatted =
                        pull_request_body_markdown_with_timezone(&pr, timezone.as_ref());
                    println!("{}", formatted.0);
                    println!("---");
                    found_prs = true;
                }
            }
            if !found_prs {
                println!("No pull requests found for the provided URLs.");
            }
        }
    }

    Ok(())
}

/// Handle get repositories command
async fn handle_get_repositories_command(
    repository_urls: Vec<RepositoryUrl>,
    format: &OutputFormat,
    github_token: &Option<String>,
    timezone: &Option<TimezoneOffset>,
    request_timeout: Option<Duration>,
    showing_release_limit: Option<usize>,
    showing_milestone_limit: Option<usize>,
) -> Result<()> {
    let github_client = GitHubClient::new(github_token.clone(), request_timeout)
        .map_err(|e| anyhow::anyhow!("Failed to create GitHub client: {}", e))?;

    let repositories =
        functions::repository::get_multiple_repository_details(&github_client, repository_urls)
            .await?;

    // Output results
    match format {
        OutputFormat::Json => {
            let json_output = serde_json::to_string_pretty(&repositories)?;
            println!("{}", json_output);
        }
        OutputFormat::Markdown => {
            if repositories.is_empty() {
                println!("No repositories found for the provided URLs.");
            } else {
                for repo in repositories {
                    let markdown_content = repository_body_markdown_with_timezone(
                        &repo,
                        timezone.as_ref(),
                        showing_release_limit,
                        showing_milestone_limit,
                    );
                    println!("{}", markdown_content.0);
                }
            }
        }
    }

    Ok(())
}

/// Handle get projects command
async fn handle_get_projects_command(
    project_urls: Vec<ProjectUrl>,
    format: &OutputFormat,
    github_token: &Option<String>,
    timezone: &Option<TimezoneOffset>,
    request_timeout: Option<Duration>,
) -> Result<()> {
    let github_client = GitHubClient::new(github_token.clone(), request_timeout)
        .map_err(|e| anyhow::anyhow!("Failed to create GitHub client: {}", e))?;

    let projects = functions::project::get_projects_details(&github_client, project_urls)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get project details: {}", e))?;

    // Output results
    match format {
        OutputFormat::Json => {
            let json_output = serde_json::to_string_pretty(&projects)?;
            println!("{}", json_output);
        }
        OutputFormat::Markdown => {
            if projects.is_empty() {
                println!("No projects found for the provided URLs.");
            } else {
                for project in projects {
                    let markdown_content =
                        project_body_markdown_with_timezone(&project, timezone.as_ref());
                    println!("{}", markdown_content.0);
                    println!("---");
                }
            }
        }
    }

    Ok(())
}
