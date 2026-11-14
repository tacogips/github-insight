use crate::github::error::ApiRetryableError;

/// Classifies GraphQL errors for retry handling.
///
/// # Arguments
///
/// * `error_msg` - The GraphQL error message to classify
///
/// # Returns
///
/// Returns an ApiRetryableError with appropriate classification.
pub fn classify_graphql_error(error_msg: &str) -> ApiRetryableError {
    use crate::github::error::ApiRetryableError;

    // Check for specific GraphQL errors that should be retried
    if error_msg.contains("A query attribute must be specified and must be a string") {
        // This error can occur due to transient query construction issues
        tracing::warn!(
            "GraphQL query construction error - will retry: {}",
            error_msg
        );
        ApiRetryableError::Retryable(format!("GraphQL query construction error: {}", error_msg))
    } else if error_msg.contains("rate limit") || error_msg.contains("API rate limit") {
        // Rate limit errors should be retried with backoff
        tracing::warn!("GraphQL rate limit error - will retry: {}", error_msg);
        ApiRetryableError::RateLimit
    } else if error_msg.contains("timeout") || error_msg.contains("server error") {
        // Server-side errors should be retried
        tracing::warn!("GraphQL server error - will retry: {}", error_msg);
        ApiRetryableError::Retryable(format!("GraphQL server error: {}", error_msg))
    } else if error_msg.contains("Could not resolve to a PullRequest")
        || error_msg.contains("Could not resolve to an Issue")
    {
        // These indicate non-existent resources, should not be retried but handled gracefully
        tracing::info!(
            "GraphQL resource not found - treating as non-retryable: {}",
            error_msg
        );
        ApiRetryableError::NonRetryable(format!("Resource not found: {}", error_msg))
    } else if error_msg.contains("Expected NAME")
        || error_msg.contains("Expected one of SCHEMA, SCALAR")
    {
        // These specific GraphQL parsing errors can be transient - retry them
        tracing::warn!("GraphQL parsing error - will retry: {}", error_msg);
        ApiRetryableError::Retryable(format!("GraphQL parsing error: {}", error_msg))
    } else if error_msg.contains("validation") || error_msg.contains("syntax") {
        // Query validation errors are typically client-side issues
        tracing::error!("GraphQL validation error - not retryable: {}", error_msg);
        ApiRetryableError::NonRetryable(format!("GraphQL validation error: {}", error_msg))
    } else {
        // Default to retryable for unknown GraphQL errors to improve reliability
        tracing::warn!(
            "Unknown GraphQL error - treating as retryable: {}",
            error_msg
        );
        ApiRetryableError::Retryable(format!("GraphQL error: {}", error_msg))
    }
}
