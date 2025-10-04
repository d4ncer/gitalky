pub mod error;
pub mod git;

// Re-export commonly used types for convenience
pub use error::{GitError, Result};
pub use git::{GitVersion, Repository, RepositoryState};
