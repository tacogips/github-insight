# Tests

This directory contains tests for the GitHub Insight MCP server.

## GitHub Client Tests

- `pull_requests.rs`: Tests for fetching pull requests by number from real GitHub repositories

## Running the Tests

### Prerequisites

1. Set the `GITHUB_INSIGHT_GITHUB_TOKEN` environment variable with a valid GitHub token:
   ```bash
   export GITHUB_INSIGHT_GITHUB_TOKEN=ghp_your_token_here
   ```

2. Run tests:
   ```bash
   cargo test
   ```

### Running Specific Tests

To run only the pull request tests:
```bash
cargo test pull_requests
```

To run a specific test function:
```bash
cargo test test_fetch_multiple_pull_requests_by_numbers
```

### Test Configuration

The tests use famous public repositories to ensure reliability:
- `rust-lang/rust` - For testing single PR fetching
- `microsoft/TypeScript` - For testing different repository types
- `facebook/react` - For testing multiple PR fetching

All tests are marked with `#[serial]` to prevent rate limiting issues with the GitHub API.

## Test Features

The tests verify:
- ✅ Fetching single pull requests by number
- ✅ Fetching multiple pull requests by numbers
- ✅ Handling non-existent pull requests gracefully
- ✅ Handling empty input correctly
- ✅ Proper error handling and timeout configuration
- ✅ Validation of pull request metadata (title, dates, etc.)

## Notes

- Tests require network access to github.com
- Tests use real GitHub API endpoints
- Rate limiting is handled through the `serial_test` crate
- A GitHub token is recommended but not strictly required for public repositories