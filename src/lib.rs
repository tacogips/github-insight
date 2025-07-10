/// Output formatting utilities for JSON and Markdown representations
pub mod formatter;

/// GitHub API client implementations and utilities for fetching repository data
pub mod github;

/// Core services for search, synchronization, and embeddings generation
pub mod services;

/// MCP tool implementations exposing library functionality through the protocol
pub mod tools;

/// Transport layer implementations for MCP server modes (stdio, SSE)
pub mod transport;

/// Core type definitions and domain models used throughout the library
pub mod types;
