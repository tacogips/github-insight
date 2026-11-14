# CLAUDE.md

Guidance for Claude Code when working with this codebase.

## Response Rules

**ALL responses throughout the session must follow these rules:**

- Always respond in English regardless of user's language
- For first response only: Start with "I will continue thinking and providing output in English."
- For first response only: Acknowledge reading CLAUDE.md
- For every response: When user provides non-English instructions, begin with "Your instruction is {corrected English translation}"
- For every response: When mentioning cargo commands, declare they use CARGO_TERM_QUIET=true

### Task Management

**Special Commands**:

- "show tasks/todos" → Display current session TODOs
- "show plan" → Display planned tasks
- "continue tasks" → Start working on pending tasks

## How to change code

**MANDATORY** Never delete tests without deliberation

### Pre-modification Review Check

1. Search for review comments in modification area (`REV-` prefix)
2. Check commit history for review context
3. Respect review guidance unless functional changes require deviation

### Required Steps

1. Follow code style guidelines
2. Update documentation (//!, ///) and verify: `CARGO_TERM_QUIET=true cargo doc --no-deps`
3. **Commit Only When Requested**: Commits should only be made when explicitly instructed by the user:

   a) Stage files (`git add`)
   b) Show summary with message and diff stats
   c) Execute commit
   d) Show result

**Summary Format**: Files, TODOs, Future Plan, commit message
**Color Coding**: 🔴 Deletions, 🟢 Additions/Modifications, 🟡 Renames

**IMPORTANT**: Never skip steps unless explicitly told.

### Test Guidelines

- Fix code to make tests pass (never remove test cases)
- Add tests for new functionality
- Maintain test coverage and strictness
- Use designated test resources:
  - Repository: https://github.com/tacogips/gitcodes-mcp-test-1
  - Project: https://github.com/users/tacogips/projects/1
- Include meaningful assertions with appropriate timeouts

### Communication Guidelines

- Seek clarification for ambiguous instructions
- Understand user goals before implementation
- Present options when unsure about details

### Git Commit Policy

- Follow commit message format from this document
- Auto-proceed without user confirmation
- **NO Claude Code attribution** - commits appear as user-made only
- Execute all "How to change code" steps above before completing

### Git Commit Message Format

**Structure** (9 sections):

1. **Objective**: Purpose, goals, challenges addressed
2. **Primary Changes**: Main changes and intent
3. **Key Technical Concepts**: Technologies, frameworks involved
4. **Files and Code Sections**: Modified files with summaries
5. **Problem Solving**: Issues addressed (include bug reproduction for fixes)
6. **Impact**: Effect on overall project
7. **Related Commits**: `Related: abc123d, def456a`
8. **Unresolved TODOs**: `- [ ]` format
9. **Future Plan**: `- [ ]` format

**Bug Fix Rule**: Always include reproduction method in commit log with fictional values for sensitive data.

## Build & Run Commands

- Build: `cargo build`
- Run: `cargo run`
- Release: `cargo build --release`
- Test: `cargo test [test_name]`
- Lint: `cargo clippy`
- Format: `cargo fmt`

## MCP Server Usage

### Running Server

```bash
# STDIO mode (for Claude Desktop integration)
github-insight-mcp stdio [--github-token TOKEN] [--timezone TIMEZONE] [--profile PROFILE] [--debug]

# HTTP mode (for web-based access and testing)
github-insight-mcp http [--address ADDR] [--github-token TOKEN] [--timezone TIMEZONE] [--profile PROFILE] [--debug]
```

### Claude Desktop Integration

```json
{
  "mcpServers": {
    "github-insight": {
      "command": "/path/to/github-insight-mcp",
      "args": ["stdio", "--timezone", "America/New_York", "--profile", "work"],
      "env": { "GITHUB_INSIGHT_GITHUB_TOKEN": "ghp_token" }
    }
  }
}
```

### Quiet Cargo Commands

Use `CARGO_TERM_QUIET=true` prefix to reduce output.

## MCP Tools

### Available Tools

#### 1. get_project_resources
Get all project resources from specified project(s). Returns all project resources as markdown array including title, description, resource counts, and timestamps. Each project resource includes field IDs that can be used for project field updates. This tool fetches all resources without pagination.

Examples:
- Get all project resources from all projects in profile: `{}`
- Get resources from specific project: `{"project_url": "https://github.com/users/username/projects/1"}`
- Get resources with light format: `{"output_option": "light"}`
- Get resources with rich format (default): `{"output_option": "rich"}`

#### 2. get_issues_details
Get issues by their URLs from specified repositories. Returns detailed issue information including comments, formatted as markdown with comprehensive details including title, body, labels, assignees, creation/update dates, and all comments with timestamps.

Examples:
- Get specific issues: `{"issue_urls": ["https://github.com/rust-lang/rust/issues/12345", "https://github.com/tokio-rs/tokio/issues/5678"]}`

#### 3. get_pull_request_details
Get pull requests by their URLs from specified repositories. Returns detailed pull request information including comments, formatted as markdown with comprehensive details including title, body, labels, assignees, creation/update dates, review status, and all comments with timestamps.

Examples:
- Get specific pull requests: `{"pull_request_urls": ["https://github.com/rust-lang/rust/pull/98765", "https://github.com/tokio-rs/tokio/pull/4321"]}`

#### 4. get_pull_request_code_diff
Get pull request code diffs by their URLs. Returns complete unified diff format for each pull request using GitHub REST API. The diff includes all file changes in standard unified diff format, making it suitable for code review and analysis.

Examples:
- Get specific pull request diffs: `{"pull_request_urls": ["https://github.com/rust-lang/rust/pull/98765", "https://github.com/tokio-rs/tokio/pull/4321"]}`

#### 5. get_project_details
Get project details by their URLs. Returns detailed project information formatted as markdown with comprehensive metadata including title, description, creation/update dates, project node ID, and other project properties. The project node ID can be used for project updates.

Examples:
- Get specific projects: `{"project_urls": ["https://github.com/users/username/projects/1", "https://github.com/orgs/orgname/projects/5"]}`

#### 6. get_repository_details
Get repository details by URLs. Returns detailed repository information formatted as markdown with comprehensive metadata including URL, description, default branch, mentionable users, labels, milestones, releases (with configurable limit), and timestamps.

Examples:
- Get all repositories from profile: `{}`
- Get specific repositories: `{"repository_urls": ["https://github.com/rust-lang/rust", "https://github.com/tokio-rs/tokio"]}`
- Get repositories with custom release limit: `{"repository_urls": ["https://github.com/rust-lang/rust"], "showing_release_limit": 5}`

#### 7. search_in_repositories
Search across multiple repositories for issues, PRs, and projects. Comprehensive search across multiple resource types with support for specific repository targeting and advanced pagination.

Examples:
- Search in specific repositories: `{"github_search_query": "memory leak", "repository_urls": ["https://github.com/rust-lang/rust", "https://github.com/tokio-rs/tokio"]}`
- Search with default query: `{"repository_urls": ["https://github.com/tokio-rs/tokio"]}`
- Search with light format: `{"github_search_query": "async await", "repository_urls": ["https://github.com/tokio-rs/tokio"], "output_option": "light", "limit": 20}`

#### 8. list_repository_urls_in_current_profile
List all repository URLs registered in the current profile. Returns repository IDs and URLs for repositories managed by the profile.

Examples:
- List all repository URLs in current profile: `{}`

#### 9. list_project_urls_in_current_profile
List all project URLs registered in the current profile. Returns project IDs and URLs for projects managed by the profile.

Examples:
- List all project URLs in current profile: `{}`

#### 9. Repository Branch Group Management Tools

Repository branch groups are collections of branches that enable organized management of related branches across multiple repositories. For example, you can group all 'feature-x' branches across different repositories, or collect all 'main' branches for release management.

**Terminology**: A "branch" refers to a repository URL and branch name pair in the format "repo_url@branch_name". For example, "https://github.com/owner/repo@main" is considered one branch.

##### register_repository_branch_group
Create a new repository branch group with branches.

Parameters:
- `profile_name`: Profile to register the group to (e.g., 'default', 'work')
- `group_name`: Optional group name (auto-generated if not provided)
- `pairs`: Array of branch specifiers in format "repo_url@branch"
- `description`: Optional description for the group

Examples:
- `{"profile_name": "default", "group_name": "feature-auth", "pairs": ["https://github.com/owner/frontend@feature-auth", "https://github.com/owner/backend@feature-auth"], "description": "Authentication feature implementation across repositories"}`
- `{"profile_name": "work", "pairs": ["https://github.com/company/api@main", "https://github.com/company/web@main"], "description": "Production release branches"}`
- `{"profile_name": "default", "group_name": "hotfix-security", "pairs": ["https://github.com/owner/backend@hotfix-security"]}`

Output: Returns the final group name as JSON string.

##### show_repository_branch_groups
List all repository branch groups in a profile.

Parameters:
- `profile_name`: Profile to list groups from

Examples:
- `{"profile_name": "default"}`

Output: Returns markdown formatted list with profile name and all group names.

##### get_repository_branch_group
Show detailed information about a specific repository branch group.

Parameters:
- `profile_name`: Profile containing the group
- `group_name`: Group name to show details for

Examples:
- `{"profile_name": "default", "group_name": "feature-auth"}`

Output: Returns markdown with group name, creation timestamp, and all branches in format "repository_url | branch:branch_name".

##### add_branch_to_branch_group
Add branches to an existing group.

Parameters:
- `profile_name`: Profile containing the group
- `group_name`: Group to add branches to
- `branch_specifiers`: Array of "repo_url@branch" specifiers

Examples:
- `{"profile_name": "default", "group_name": "feature-auth", "branch_specifiers": ["https://github.com/owner/mobile@feature-auth"]}`

##### remove_branch_from_branch_group
Remove branches from a group.

Parameters:
- `profile_name`: Profile containing the group
- `group_name`: Group to remove branches from
- `branch_specifiers`: Array of "repo_url@branch" specifiers

Examples:
- `{"profile_name": "default", "group_name": "feature-auth", "branch_specifiers": ["https://github.com/owner/mobile@feature-auth"]}`

##### unregister_repository_branch_group
Remove a repository branch group completely.

Parameters:
- `profile_name`: Profile containing the group
- `group_name`: Group to remove

Examples:
- `{"profile_name": "default", "group_name": "old-feature"}`

Output: Returns JSON with removed group information including all branches.

##### rename_repository_branch_group
Change a group's name while preserving its contents.

Parameters:
- `profile_name`: Profile containing the group
- `old_name`: Current group name
- `new_name`: New group name

Examples:
- `{"profile_name": "default", "old_name": "temp-feature", "new_name": "release-v2"}`

##### cleanup_repository_branch_groups
Remove groups older than specified days.

Parameters:
- `profile_name`: Profile to clean up
- `days`: Age threshold in days

Examples:
- `{"profile_name": "default", "days": 30}`

Output: Returns JSON array of removed groups with their details.

### Common Workflows

1. **Profile Management**:
   - Use list_repository_urls_in_current_profile to get all repository URLs registered in the current profile
   - Use list_project_urls_in_current_profile to get all project URLs registered in the current profile

2. **Repository Search**:
   - Use search_in_repositories to find issues/PRs by keywords across specific repositories
   - Support for pagination using cursors for large result sets
   - Choose between light and rich output formats

3. **Specific Resource Access**:
   - Use get_issues_details to get detailed issue information with comments
   - Use get_pull_request_details to get detailed pull request information with comments and code review threads
   - Use get_pull_request_code_diff to get complete unified diff for code changes

4. **Project Management**:
   - Use get_project_resources to access project boards and associated resources
   - Fetch from all projects in profile or specific project URLs
   - Choose between light and rich output formats (default: rich)

5. **Repository Branch Group Management**:
   - Use register_repository_branch_group to create collections of related branches
   - Use show_repository_branch_groups to list all groups in a profile
   - Use get_repository_branch_group to view detailed information about a specific group
   - Use add_branch_to_branch_group and remove_branch_from_branch_group to modify group membership
   - Use cleanup_repository_branch_groups for automated maintenance of old groups

6. **Output Formatting**:
   - Rich format provides comprehensive details including full comments, timestamps, custom fields
   - Light format provides minimal information for quick overview
   - get_project_resources defaults to rich format for detailed project information
   - search_in_repositories defaults to light format for quick search results

## CLI Usage

### Running CLI

```bash
# Basic usage
cargo run --bin github-insight-cli -- [OPTIONS] <COMMAND>

# With environment variables
GITHUB_INSIGHT_GITHUB_TOKEN=ghp_token cargo run --bin github-insight-cli -- [OPTIONS] <COMMAND>
```

### Global Options

- `--format <FORMAT>`: Output format (json, markdown) [default: markdown]
- `--github-token <GITHUB_TOKEN>`: GitHub personal access token
- `--timezone <TIMEZONE>`: Timezone for datetime formatting (e.g., "JST", "+09:00", "America/New_York", "UTC")
- `--request-timeout <REQUEST_TIMEOUT>`: Request timeout in seconds [default: 30]

### Commands

#### Profile Management

- `create-profile`: Create a new profile for organizing repositories and projects with optional description
- `delete-profile`: Delete a profile and all its associated repository and project registrations (irreversible)
- `list-profiles`: Display all available profiles with their configurations and metadata

#### Repository Management

- `register-repo`: Register a repository to a profile for centralized management and search operations
- `unregister-repo`: Remove a repository from a profile, excluding it from search and management operations
- `list-repos`: Display all repositories registered in a specific profile with their URLs and registration details

#### Project Management

- `register-project`: Register a GitHub project to a profile for comprehensive resource management and tracking with pagination support
- `unregister-project`: Remove a GitHub project from a profile, excluding it from resource management and tracking
- `list-projects`: Display all GitHub projects registered in a specific profile with their URLs and metadata

#### Repository Branch Group Management

- `register-group`: Create a new repository branch group with branches and optional description in specified profile
- `unregister-group`: Remove a repository branch group completely from specified profile
- `list-branch-groups`: Display all repository branch groups in a specified profile
- `show-group`: Show detailed information about a specific repository branch group
- `add-branch-to-branch-group`: Add branches to an existing group
- `remove-branch-from-branch-group`: Remove branches from an existing group
- `rename-group`: Change a group's name while preserving its contents
- `cleanup-groups`: Remove groups older than specified days from specified profile

#### Data Operations

- `search`: Search for issues and pull requests across multiple repositories with advanced GitHub search syntax and pagination support. Use `get-issues` and `get-pull-requests` commands to get more detailed information. Note: Repository specifications (repo:owner/name) within the query are not supported and will be ignored - repository filtering is handled by the --repository-url option (expects full GitHub URL format) and registered repositories in the profile
- `get-project-resources`: Fetch detailed project resources including items, metadata, timestamps, and assignees with comprehensive pagination support. Supports light/rich output format (default: rich). Use `get-issues` and `get-pull-requests` commands to get more detailed information
- `get-issues`: Fetch detailed issue information including comments, metadata, labels, and timeline events by URLs (formatted as markdown with comprehensive details)
- `get-pull-requests`: Fetch detailed pull request information including comments, metadata, reviews, and timeline events by URLs (formatted as markdown with comprehensive details)
- `get-repositories`: Fetch detailed repository information including metadata, statistics, releases (with configurable limit using --showing-release-limit, default: 10), milestones (with configurable limit using --showing-milestone-limit, default: 10), and configuration by URLs (formatted as markdown with comprehensive details)
- `get-projects`: Fetch detailed project information including metadata, description, and timestamps by URLs (formatted as markdown with comprehensive details)

#### General

- `help`: Print help message or help for specific subcommands

## Code Style Guidelines

- Rust 2024 edition, rustfmt default settings
- snake_case naming, prefer Result<T,E> over unwrap()
- Organize imports: std first, then external crates
- Use structured logging (env_logger, tracing)
- Type annotations for public functions
- Effective ownership system (avoid unnecessary clones)
- Module declaration order: `pub mod`, `mod`, `pub use`, `use` (each block separated by blank line)
- Place structs/enums with their implementations together

### Implementation Principles

- **Divide and Conquer**: Apply divide and conquer approach to break down complex implementations into appropriate methods and split files as needed for maintainability
- **DRY (Don't Repeat Yourself)**: Properly consolidate common processing logic and avoid recreating existing shared functionality. Always check for existing common utilities before implementing new ones
- **Code Reusability**: Identify and extract reusable components into separate modules or functions to promote code sharing across the codebase
- **Modular Design**: Structure code into logical modules with clear responsibilities and minimal coupling between components

### General Coding Principles

**SOLID Principles**:

- **Single Responsibility**: Each function/struct should have one reason to change
- **Open/Closed**: Open for extension, closed for modification
- **Liskov Substitution**: Subtypes must be substitutable for their base types
- **Interface Segregation**: Clients shouldn't depend on interfaces they don't use
- **Dependency Inversion**: Depend on abstractions, not concretions

**Clean Code Practices**:

- **Meaningful Names**: Use descriptive, searchable names that express intent
- **Small Functions**: Functions should do one thing well (max 20-30 lines)
- **Function Arguments**: Minimize argument count (ideally 0-2, maximum 3)
- **Comments**: Code should be self-documenting; comments explain "why", not "what"
- **Consistent Formatting**: Follow project's formatting standards religiously

**Error Handling & Robustness**:

- **Fail Fast**: Detect and report errors as early as possible
- **Graceful Degradation**: System should continue operating with reduced functionality
- **Input Validation**: Always validate inputs at system boundaries
- **Resource Management**: Properly handle resource cleanup (RAII pattern)
- **Defensive Programming**: Assume inputs can be malicious or malformed

**Performance & Efficiency**:

- **Premature Optimization**: Avoid unless profiling shows actual bottlenecks
- **Big O Awareness**: Understand algorithmic complexity of your solutions
- **Memory Management**: Minimize allocations, avoid memory leaks
- **Lazy Loading**: Load resources only when needed
- **Caching Strategy**: Cache expensive operations with appropriate invalidation

**Testing & Quality**:

- **Test-Driven Development**: Write tests before implementation when possible
- **Test Coverage**: Aim for high coverage but focus on critical paths
- **Test Isolation**: Tests should be independent and repeatable
- **Mock External Dependencies**: Use mocks for external services/APIs
- **Integration Tests**: Test the complete flow of critical features

**Security & Safety**:

- **Principle of Least Privilege**: Grant minimum necessary permissions
- **Input Sanitization**: Never trust user input; validate and sanitize
- **Secure Defaults**: Use secure configurations by default
- **Secrets Management**: Never hardcode credentials; use environment variables
- **Audit Trail**: Log security-relevant events for monitoring

### Rust-Specific Guidelines

**Error Handling & Reliability**:

- Always use `Result<T, E>` instead of `unwrap()` or `panic!()`
- Implement proper error propagation using `?` operator
- Use `anyhow` for error handling in application code
- Handle all async operations with proper timeout handling

**Performance & Memory**:

- Avoid unnecessary `clone()` operations - use references when possible
- Use `ahash` for HashMap operations (already in dependencies)
- Implement pagination for large data sets
- Use streaming for large API responses

**Async Programming**:

- Use tokio runtime features appropriately
- Implement proper cancellation handling
- Use `futures` utilities for complex async operations
- Apply timeouts to external API calls

**Security**:

- Never log sensitive data (tokens, personal information)
- Use `rustls` for TLS connections
- Validate all external inputs
- Use secure defaults for HTTP clients

**Testing & Documentation**:

- Write unit tests for all public functions
- Use `mockito` for HTTP mocking in tests
- Document all public APIs with `///` comments
- Use `serial_test` for tests requiring isolation

**GitHub API Integration**:

- Use `octocrab` client consistently
- Implement proper rate limiting
- Handle GraphQL pagination correctly
- Cache responses appropriately using `tantivy`

**Configuration Management**:

- Use `toml` for configuration files
- Support environment variables
- Implement profile-based configuration
- Use `dirs` for platform-specific paths
