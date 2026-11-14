/// Classification of API errors for retry logic
#[derive(Debug, Clone, PartialEq)]
pub enum ApiRetryableError {
    /// Errors that should be retried (5xx server errors, network issues)
    Retryable(String),
    /// Rate limiting errors (429) - retryable with backoff
    RateLimit,
    /// Client errors that should not be retried (4xx except 429)
    NonRetryable(String),
}

impl ApiRetryableError {
    /// Convert octocrab error to appropriate retry category
    pub fn from_octocrab_error(error: octocrab::Error) -> Self {
        // Log the raw error for debugging
        tracing::debug!("Raw octocrab error: {:?}", error);

        let result = match &error {
            // Handle different error types based on actual octocrab Error variants
            octocrab::Error::GitHub { source, .. } => {
                // GitHub API returned an error response
                // Check the status code for classification
                let status = source.status_code.as_u16();
                let detailed_error = format!(
                    "GitHub API error - Status: {}, Message: {:?}, Documentation: {:?}",
                    status, source.message, source.documentation_url
                );
                tracing::error!("GitHub API error details: {}", detailed_error);

                match status {
                    429 => {
                        tracing::warn!("Rate limit (429) detected for GitHub API request");
                        Self::RateLimit
                    }
                    403 => {
                        // Check if this is a rate limit error based on the message
                        if source.message.contains("rate limit")
                            || source.message.contains("API rate limit")
                        {
                            tracing::warn!(
                                "Rate limit (403) detected for GitHub API request: {}",
                                source.message
                            );
                            Self::RateLimit
                        } else {
                            tracing::error!(
                                "Non-retryable client error ({}): {}",
                                status,
                                detailed_error
                            );
                            Self::NonRetryable(detailed_error)
                        }
                    }
                    400..=499 => {
                        tracing::error!(
                            "Non-retryable client error ({}): {}",
                            status,
                            detailed_error
                        );
                        Self::NonRetryable(detailed_error)
                    }
                    500..=599 => {
                        tracing::warn!(
                            "Server error ({}) - will retry: {}",
                            status,
                            detailed_error
                        );
                        Self::Retryable(detailed_error)
                    }
                    _ => {
                        tracing::error!(
                            "Unknown status code ({}) - treating as non-retryable: {}",
                            status,
                            detailed_error
                        );
                        Self::NonRetryable(detailed_error)
                    }
                }
            }
            octocrab::Error::Http { .. } => {
                // HTTP layer error - likely retryable
                let error_msg = format!("HTTP layer error: {}", error);
                tracing::warn!("HTTP layer error - will retry: {}", error_msg);
                Self::Retryable(error_msg)
            }
            octocrab::Error::Hyper { .. } => {
                // Lower level HTTP error - likely retryable
                let error_msg = format!("Hyper HTTP error: {}", error);
                tracing::warn!("Hyper error - will retry: {}", error_msg);
                Self::Retryable(error_msg)
            }
            octocrab::Error::Json { .. } => {
                // JSON parsing error - not retryable
                let error_msg = format!("JSON parsing error: {}", error);
                tracing::error!("JSON parsing error - not retryable: {}", error_msg);
                Self::NonRetryable(error_msg)
            }
            octocrab::Error::Uri { .. } => {
                // URI parsing error - not retryable
                let error_msg = format!("URI parsing error: {}", error);
                tracing::error!("URI parsing error - not retryable: {}", error_msg);
                Self::NonRetryable(error_msg)
            }
            _ => {
                // Unknown error type - default to non-retryable for safety
                let error_msg = format!("Unknown error type: {}", error);
                tracing::error!(
                    "Unknown error type - treating as non-retryable: {}",
                    error_msg
                );
                Self::NonRetryable(error_msg)
            }
        };

        tracing::debug!(
            "Error classification result: {:?} for error: {}",
            result,
            error
        );
        result
    }
}

impl std::fmt::Display for ApiRetryableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Retryable(msg) => write!(f, "Retryable error: {}", msg),
            Self::RateLimit => write!(f, "Rate limit error"),
            Self::NonRetryable(msg) => write!(f, "Non-retryable error: {}", msg),
        }
    }
}

impl std::error::Error for ApiRetryableError {}
