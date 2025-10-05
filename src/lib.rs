pub mod error;
pub mod error_translation;
pub mod git;
pub mod llm;
pub mod security;
pub mod ui;

// Re-export commonly used types for convenience
pub use error::{GitError, Result};
pub use error_translation::{ErrorTranslator, UserFriendlyError};
pub use git::{GitVersion, Repository, RepositoryState};
pub use security::{CommandValidator, DangerousOp, ValidatedCommand, ValidationError};
