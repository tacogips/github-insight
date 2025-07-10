//! Transport layer implementations for MCP server
//!
//! This module provides different transport mechanisms for running
//! the MCP server, including stdio and SSE (Server-Sent Events).

/// SSE (Server-Sent Events) transport for HTTP-based MCP communication
pub mod sse_server;

/// Standard I/O transport for subprocess-based MCP communication
pub mod stdio;
