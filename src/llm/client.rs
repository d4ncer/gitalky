use crate::llm::context::RepoContext;
use async_trait::async_trait;
use thiserror::Error;

/// Errors that can occur during LLM operations
#[derive(Debug, Error)]
pub enum LLMError {
    #[error("API request failed: {0}")]
    ApiError(String),

    #[error("Rate limit exceeded, retry after {0}s")]
    RateLimitExceeded(u64),

    #[error("Request timeout")]
    Timeout,

    #[error("Invalid API response: {0}")]
    InvalidResponse(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Represents a git command with optional explanation
#[derive(Debug, Clone)]
pub struct GitCommand {
    pub command: String,
    pub explanation: Option<String>,
}

/// Trait for LLM clients that can translate natural language to git commands
#[async_trait]
pub trait LLMClient: Send + Sync {
    /// Translate a natural language query into a git command
    async fn translate(&self, query: &str, context: &RepoContext) -> Result<GitCommand, LLMError>;
}
