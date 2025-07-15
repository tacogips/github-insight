# GitHub Insight MCP Server

A powerful Model Context Protocol (MCP) server that provides AI assistants with direct access to GitHub repositories, issues, pull requests, and project data. Built in Rust for performance and reliability.

## Features

### 🔍 **Comprehensive Search**
- Search across multiple repositories simultaneously
- Support for GitHub's advanced search syntax
- Filter by state, labels, assignees, dates, and more
- Cursor-based pagination for efficient large result handling
- Intelligent query optimization based on project type detection
- Flexible output formatting (light/rich) for different use cases

### 📊 **Project Management**
- Access GitHub Projects (beta) with detailed information
- Track project progress, status, and associated resources
- Cross-reference between projects, issues, and pull requests
- Comprehensive author, assignee, and label support
- Optimized performance with intelligent query strategies

### 🎯 **Direct Resource Access**
- Fetch issues and pull requests by URL
- Get detailed information including comments, reviews, and metadata
- Access repository information and statistics
- Timeline events and comprehensive issue tracking

### 🔧 **Profile Management**
- Organize repositories and projects into profiles
- Isolate data by profile for different contexts
- CLI tools for profile and repository management
- Enhanced CLI help documentation and streamlined display

## Installation

### Prerequisites
- Rust 2024 edition
- GitHub Personal Access Token (for API access)
- Required token permissions: `repo`, `project`, `read:org`

### From Source
```bash
git clone https://github.com/tacogips/github-insight
cd github-insight
CARGO_TERM_QUIET=true cargo build --release
```

### Binaries
Two main binaries are built:
- `github-insight-mcp`: MCP server for AI assistant integration
- `github-insight-cli`: Command-line interface for direct usage

## Quick Start

### 1. Set up GitHub Token
```bash
export GITHUB_INSIGHT_GITHUB_TOKEN="ghp_your_token_here"
```

### 2. Run MCP Server
```bash
# STDIO mode (for Claude Desktop)
./target/release/github-insight-mcp stdio --github-token $GITHUB_INSIGHT_GITHUB_TOKEN

# HTTP mode (for web integrations) 
./target/release/github-insight-mcp http --address 0.0.0.0:8080 --github-token $GITHUB_INSIGHT_GITHUB_TOKEN

# Enable debug mode and sync operations
./target/release/github-insight-mcp stdio --debug --sync
```

### 3. Use CLI Tools
```bash
# Register a repository
./target/release/github-insight-cli register-repo https://github.com/rust-lang/rust

# Search for issues with advanced filters
./target/release/github-insight-cli search "memory leak" --state open --limit 50

# Get project information with JSON output
./target/release/github-insight-cli get-project-resources https://github.com/users/username/projects/1 --format json

# List all registered repositories
./target/release/github-insight-cli list-repos
```

## Claude Desktop Integration

Add to your Claude Desktop configuration:

```json
{
  "mcpServers": {
    "github-insight": {
      "command": "/path/to/github-insight-mcp",
      "args": ["stdio"],
      "env": {
        "GITHUB_INSIGHT_GITHUB_TOKEN": "ghp_your_token_here"
      }
    }
  }
}
```

## MCP Tools

### `get_project_resources`
Retrieve detailed project information including associated issues and pull requests with comprehensive pagination support. Use `get_issues_details` and `get_pull_request_details` functions to get more detailed information.

```json
// Get all projects from profile
{"project_url": null}

// Get specific project
{"project_url": "https://github.com/users/username/projects/1"}

// Get project with pagination
{
  "project_url": "https://github.com/users/username/projects/1",
  "after": "cursor_token_here",
  "first": 50
}
```

### `get_issues_details`
Fetch detailed issue information by GitHub URLs, formatted as markdown with comprehensive details including title, body, labels, assignees, creation/update dates, and all comments with timestamps.

```json
{
  "issue_urls": [
    "https://github.com/owner/repo/issues/123",
    "https://github.com/owner/repo/issues/456"
  ]
}
```

### `get_pull_request_details`
Retrieve comprehensive pull request data including reviews and commits, formatted as markdown with comprehensive details including title, body, labels, assignees, creation/update dates, review status, and all comments with timestamps.

```json
{
  "pull_request_urls": [
    "https://github.com/owner/repo/pull/123",
    "https://github.com/owner/repo/pull/456"
  ]
}
```

### `search_across_repositories`
Powerful search across multiple repositories with advanced filtering and flexible output formatting. Use `get_issues_details` and `get_pull_request_details` functions to get more detailed information.

```json
// Basic search (default light format)
{"query": "authentication bug"}

// Advanced search with filters
{
  "query": "is:open label:bug created:>2024-01-01",
  "repository_url": "https://github.com/owner/repo",
  "limit": 50
}

// Search with rich output format (comprehensive details)
{
  "query": "memory leak",
  "output_option": "rich"
}

// Search with light output format (minimal information)
{
  "query": "performance issue",
  "output_option": "light"
}

// Paginated search
{
  "query": "memory leak",
  "cursors": [
    {
      "repository_id": {"owner": "owner", "repository_name": "repo"},
      "cursor": "cursor_token_here"
    }
  ]
}
```

#### Output Format Options
- **`light`** (default): Minimal information including title, status, truncated body, and key metadata
- **`rich`**: Comprehensive details including full body, comments, labels, assignees, dates, and all metadata

## CLI Commands

### Repository Management
```bash
# Register a repository to profile
github-insight-cli register-repo https://github.com/owner/repo --profile dev

# List registered repositories
github-insight-cli list-repos --profile dev

# Remove repository from profile
github-insight-cli unregister-repo https://github.com/owner/repo --profile dev
```

### Project Management
```bash
# Register a project to profile
github-insight-cli register-project https://github.com/users/username/projects/1 --profile dev

# Get project information
github-insight-cli get-project-resources https://github.com/users/username/projects/1 --format json
```

### Search Operations
```bash
# Search across all repositories in profile
github-insight-cli search "memory leak" --state open --limit 20

# Search in specific repository
github-insight-cli search "authentication" --repository-url https://github.com/owner/repo

# Advanced search with multiple filters
github-insight-cli search "is:open label:bug created:>2024-01-01" --limit 50

# Get specific issue
github-insight-cli get-issues https://github.com/owner/repo/issues/123

# Get specific pull request
github-insight-cli get-pull-requests https://github.com/owner/repo/pull/456
```

### Profile Management
```bash
# List all profiles
github-insight-cli list-profiles

# Create new profile
github-insight-cli create-profile work

# Delete profile
github-insight-cli delete-profile old-profile
```

## Configuration

### Environment Variables
- `GITHUB_INSIGHT_GITHUB_TOKEN`: GitHub Personal Access Token
- `GITHUB_INSIGHT_PROFILE`: Default profile name
- `GITHUB_INSIGHT_CONFIG_DIR`: Custom configuration directory

### GitHub Token Permissions
Your GitHub token needs the following permissions:
- `repo`: Access to repository data and issues
- `project`: Access to GitHub Projects (beta)
- `read:org`: Access to organization projects
- `read:user`: Access to user profile information

### Profile Configuration
Profiles are stored in `~/.config/github-insight/profiles/` (or system equivalent).

## Development

### Building
```bash
CARGO_TERM_QUIET=true cargo build
```

### Testing
```bash
CARGO_TERM_QUIET=true cargo test
```

### Linting
```bash
CARGO_TERM_QUIET=true cargo clippy
```

### Documentation
```bash
CARGO_TERM_QUIET=true cargo doc --no-deps
```

### Test Resources
- Test Repository: https://github.com/tacogips/gitcodes-mcp-test-1
- Test Project: https://github.com/users/tacogips/projects/1

## Architecture

### Core Modules
- **`github`**: GitHub API client and GraphQL queries with optimized performance
- **`services`**: Business logic for search, profile management, and data fetching
- **`tools`**: MCP tool implementations with comprehensive error handling
- **`transport`**: MCP server transport layers (stdio, HTTP/SSE)
- **`formatter`**: Output formatting for markdown and JSON with streamlined display
- **`types`**: Core data structures and domain models
- **`bin`**: CLI and MCP server binaries with enhanced help documentation

### Key Technologies
- **Rust 2024**: Modern, safe systems programming
- **Tokio**: Async runtime for high-performance networking
- **Octocrab**: GitHub API client
- **rmcp**: Model Context Protocol implementation
- **Tantivy**: Full-text search engine
- **Serde**: Serialization framework

## Performance Features

- **Async/await**: Non-blocking I/O for high concurrency
- **Connection pooling**: Efficient GitHub API usage with timeout handling
- **Caching**: Intelligent caching of GitHub data using Tantivy
- **Pagination**: Cursor-based pagination for efficient large result handling
- **Rate limiting**: Respects GitHub API limits with intelligent retry logic
- **Query optimization**: Intelligent query strategy based on project type detection
- **Performance monitoring**: Comprehensive logging and debugging capabilities

## Security

- **Token security**: Tokens are never logged or exposed
- **TLS**: All connections use rustls for security
- **Input validation**: All inputs are validated and sanitized
- **Error handling**: Comprehensive error handling with anyhow

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes following the coding guidelines
4. Add tests for new functionality
5. Run the full test suite
6. Submit a pull request

### Coding Guidelines
- Follow Rust 2024 edition conventions
- Use `rustfmt` for formatting
- Run `clippy` for linting
- Write comprehensive tests
- Document public APIs

## License

MIT License - see LICENSE file for details.

## Support

For issues, questions, or contributions:
- GitHub Issues: https://github.com/tacogips/github-insight/issues
- Repository: https://github.com/tacogips/github-insight
