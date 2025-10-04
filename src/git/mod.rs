pub mod executor;
pub mod parser;
pub mod repository;
pub mod version;

// Re-export commonly used types
pub use executor::{CommandOutput, GitExecutor};
pub use parser::{
    BranchEntry, CommitEntry, FileStatus, StashEntry, StatusEntry,
    parse_branch_list, parse_log, parse_stash_list, parse_status_porcelain_v2,
};
pub use repository::{Repository, RepositoryState};
pub use version::GitVersion;
