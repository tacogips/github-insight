use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label(String);

impl Label {
    pub fn new(name: String) -> Self {
        Label(name)
    }

    pub fn name(&self) -> &str {
        &self.0
    }
}

impl From<String> for Label {
    fn from(name: String) -> Self {
        Label(name)
    }
}

impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
