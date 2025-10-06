use std::io;
use thiserror::Error;

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

    #[error("{0}")]
    Custom(String),
}

pub type Result<T> = std::result::Result<T, GitError>;
