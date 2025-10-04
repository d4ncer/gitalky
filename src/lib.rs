pub mod error;
pub mod git;
pub mod llm;
pub mod ui;

// Re-export commonly used types for convenience
pub use error::{GitError, Result};
pub use git::{GitVersion, Repository, RepositoryState};
