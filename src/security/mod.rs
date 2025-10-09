pub mod validator;

pub use validator::{CommandValidator, DangerousOp, ValidatedCommand, ValidationError};

/// Allowlist of permitted git subcommands
///
/// This list is used by both the CommandValidator (for command validation)
/// and the LLM Translator (for LLM output validation) to ensure consistency.
///
/// Adding a new subcommand requires careful security review.
pub const ALLOWED_GIT_SUBCOMMANDS: &[&str] = &[
    // Read operations
    "status",
    "log",
    "show",
    "diff",
    "branch",
    "tag",
    "remote",
    "reflog",
    "blame",
    "describe",
    // Write operations
    "add",
    "commit",
    "checkout",
    "switch",
    "restore",
    "reset",
    "revert",
    "merge",
    "rebase",
    "cherry-pick",
    "stash",
    "clean",
    // Remote operations
    "push",
    "pull",
    "fetch",
    "clone",
    // Configuration (repo-level only)
    "config",
    // Dangerous operations (require confirmation)
    "filter-branch",
];
