use std::io;
use thiserror::Error;

// Import module-level errors for AppError
use crate::config::settings::ConfigError;
use crate::config::first_run::SetupError;
use crate::llm::client::LLMError;
use crate::llm::translator::TranslationError;
use crate::security::validator::ValidationError;

/// Errors that can occur during git operations
#[derive(Debug, Error)]
pub enum GitError {
    #[error("Not a git repository")]
    NotARepository,

    #[error("Git command failed: {0}")]
    CommandFailed(String),

    #[error("Failed to parse git output: {0}")]
    ParseError(String),

    #[error("Git version {0} is too old. Minimum required: 2.20")]
    GitVersionTooOld(String),

    #[error("Failed to detect git version: {0}")]
    GitVersionDetectionFailed(String),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
}

/// Top-level application error that wraps all module-specific errors
///
/// This provides a unified error type for application-level code while preserving
/// the specific error context from each module. All module errors automatically
/// convert to AppError via the `From` trait.
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Git error: {0}")]
    Git(#[from] GitError),

    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("LLM error: {0}")]
    Llm(#[from] LLMError),

    #[error("Translation error: {0}")]
    Translation(#[from] TranslationError),

    #[error("Security validation error: {0}")]
    Security(#[from] ValidationError),

    #[error("Setup error: {0}")]
    Setup(#[from] SetupError),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
}

/// Result type for git operations
pub type GitResult<T> = std::result::Result<T, GitError>;

/// Result type for application-level operations
pub type AppResult<T> = std::result::Result<T, AppError>;
