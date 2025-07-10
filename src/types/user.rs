//! User and participant types for Git resources
//!
//! This module provides types for user identification and participation
//! in Git resources like issues and pull requests.

use serde::{Deserialize, Serialize};

/// User identifier wrapper type for GitHub usernames
///
/// This type provides type-safe user identification for GitHub users,
/// storing the username for complete identification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct User(String);

impl User {
    /// Creates a new UserId with the specified username
    pub fn new(username: String) -> Self {
        Self(username)
    }

    /// Get the username as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for User {
    fn from(s: &str) -> Self {
        User::new(s.to_string())
    }
}

impl From<String> for User {
    fn from(s: String) -> Self {
        User::new(s)
    }
}

impl PartialEq<str> for User {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for User {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}
