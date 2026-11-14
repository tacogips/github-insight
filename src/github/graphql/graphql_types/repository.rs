use serde::{Deserialize, Serialize};

/// Wrapper type for milestone numbers providing type safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MilestoneNumber(pub u64);

impl MilestoneNumber {
    /// Create a new milestone number
    pub fn new(number: u64) -> Self {
        Self(number)
    }

    /// Get the inner value
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for MilestoneNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub owner: RepositoryOwner,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryOwner {
    pub login: String,
}
