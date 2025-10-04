use crate::error::{GitError, Result};
use crate::git::executor::GitExecutor;
use crate::git::parser::{self, CommitEntry, StashEntry, StatusEntry};
use std::env;
use std::path::{Path, PathBuf};

/// Represents a git repository and provides access to its state
#[derive(Debug)]
pub struct Repository {
    path: PathBuf,
    executor: GitExecutor,
}

impl Repository {
    /// Detect git repository from current working directory
    pub fn discover() -> Result<Self> {
        let current_dir = env::current_dir().map_err(|e| {
            GitError::IoError(e)
        })?;

        Self::discover_from(&current_dir)
    }

    /// Detect git repository starting from a specific directory
    pub fn discover_from<P: AsRef<Path>>(start_path: P) -> Result<Self> {
        let mut current = start_path.as_ref().to_path_buf();

        loop {
            let git_dir = current.join(".git");
            if git_dir.exists() {
                return Ok(Self::new(current));
            }

            // Move up to parent directory
            if !current.pop() {
                return Err(GitError::NotARepository);
            }
        }
    }

    /// Create a Repository for a known git directory
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref().to_path_buf();
        let executor = GitExecutor::new(&path);

        Self { path, executor }
    }

    /// Get the repository path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Query the current repository state
    pub fn state(&self) -> Result<RepositoryState> {
        let current_branch = self.current_branch()?;
        let upstream = self.upstream_info(&current_branch)?;
        let status_entries = self.status()?;
        let commits = self.recent_commits(10)?;
        let stashes = self.stash_list()?;

        // Categorize status entries
        let mut staged = Vec::new();
        let mut unstaged = Vec::new();
        let mut untracked = Vec::new();

        for entry in status_entries {
            if entry.staged {
                staged.push(entry.clone());
            }
            if entry.unstaged {
                unstaged.push(entry.clone());
            }
            if entry.status == parser::FileStatus::Untracked {
                untracked.push(entry);
            }
        }

        // Detect special states
        let in_merge = self.path.join(".git/MERGE_HEAD").exists();
        let in_rebase = self.path.join(".git/rebase-merge").exists()
            || self.path.join(".git/rebase-apply").exists();

        Ok(RepositoryState {
            current_branch,
            upstream,
            staged_files: staged,
            unstaged_files: unstaged,
            untracked_files: untracked,
            recent_commits: commits,
            stashes,
            in_merge,
            in_rebase,
        })
    }

    /// Get the current branch name
    fn current_branch(&self) -> Result<Option<String>> {
        match self.executor.execute("branch --show-current") {
            Ok(output) => {
                let branch = output.stdout.trim();
                if branch.is_empty() {
                    // Detached HEAD state
                    Ok(None)
                } else {
                    Ok(Some(branch.to_string()))
                }
            }
            Err(_) => Ok(None),
        }
    }

    /// Get upstream tracking info for the current branch
    fn upstream_info(&self, branch: &Option<String>) -> Result<Option<UpstreamInfo>> {
        let branch_name = match branch {
            Some(b) => b,
            None => return Ok(None), // No branch (detached HEAD)
        };

        // Get upstream branch name
        let cmd = format!(
            "for-each-ref --format=%(upstream:short) refs/heads/{}",
            branch_name
        );
        let upstream_branch = match self.executor.execute(&cmd) {
            Ok(output) => {
                let upstream = output.stdout.trim();
                if upstream.is_empty() {
                    return Ok(None); // No upstream configured
                }
                upstream.to_string()
            }
            Err(_) => return Ok(None),
        };

        // Get ahead/behind counts
        let cmd = format!("rev-list --left-right --count {}...{}", branch_name, upstream_branch);
        match self.executor.execute(&cmd) {
            Ok(output) => {
                let parts: Vec<&str> = output.stdout.split_whitespace().collect();
                if parts.len() == 2 {
                    let ahead = parts[0].parse::<usize>().unwrap_or(0);
                    let behind = parts[1].parse::<usize>().unwrap_or(0);

                    Ok(Some(UpstreamInfo {
                        remote_branch: upstream_branch,
                        ahead,
                        behind,
                    }))
                } else {
                    Ok(None)
                }
            }
            Err(_) => Ok(None),
        }
    }

    /// Get status entries
    fn status(&self) -> Result<Vec<StatusEntry>> {
        let output = self.executor.execute("status --porcelain=v2")?;
        parser::parse_status_porcelain_v2(&output.stdout)
    }

    /// Get recent commits
    fn recent_commits(&self, count: usize) -> Result<Vec<CommitEntry>> {
        let cmd = format!("log -n {} --format=%H%x00%s", count);
        match self.executor.execute(&cmd) {
            Ok(output) => parser::parse_log(&output.stdout),
            Err(_) => Ok(Vec::new()), // Empty repo has no commits
        }
    }

    /// Get stash list
    fn stash_list(&self) -> Result<Vec<StashEntry>> {
        match self.executor.execute("stash list --format=%gd%x00%s") {
            Ok(output) => parser::parse_stash_list(&output.stdout),
            Err(_) => Ok(Vec::new()), // No stashes
        }
    }

    /// Get the git executor for this repository
    pub fn executor(&self) -> &GitExecutor {
        &self.executor
    }
}

/// Upstream tracking information
#[derive(Debug, Clone)]
pub struct UpstreamInfo {
    pub remote_branch: String,
    pub ahead: usize,
    pub behind: usize,
}

/// Represents the current state of a git repository
#[derive(Debug, Clone)]
pub struct RepositoryState {
    pub current_branch: Option<String>,
    pub upstream: Option<UpstreamInfo>,
    pub staged_files: Vec<StatusEntry>,
    pub unstaged_files: Vec<StatusEntry>,
    pub untracked_files: Vec<StatusEntry>,
    pub recent_commits: Vec<CommitEntry>,
    pub stashes: Vec<StashEntry>,
    pub in_merge: bool,
    pub in_rebase: bool,
}

impl RepositoryState {
    /// Check if the repository is in a clean state (no changes)
    pub fn is_clean(&self) -> bool {
        self.staged_files.is_empty()
            && self.unstaged_files.is_empty()
            && self.untracked_files.is_empty()
    }

    /// Check if in detached HEAD state
    pub fn is_detached(&self) -> bool {
        self.current_branch.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    fn create_test_repo() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().to_path_buf();

        // Initialize git repo
        Command::new("git")
            .args(&["init"])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        // Configure git
        Command::new("git")
            .args(&["config", "user.name", "Test User"])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        Command::new("git")
            .args(&["config", "user.email", "test@example.com"])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        (temp_dir, repo_path)
    }

    #[test]
    fn test_discover_repo() {
        let (_temp, repo_path) = create_test_repo();

        // Change to repo directory
        let original_dir = env::current_dir().unwrap();
        env::set_current_dir(&repo_path).unwrap();

        let repo = Repository::discover().unwrap();

        // Canonicalize both paths for comparison (handles symlinks on macOS)
        let repo_canonical = repo.path().canonicalize().unwrap();
        let expected_canonical = repo_path.canonicalize().unwrap();
        assert_eq!(repo_canonical, expected_canonical);

        // Restore original directory
        env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_discover_from_subdirectory() {
        let (_temp, repo_path) = create_test_repo();

        // Create subdirectory
        let sub_dir = repo_path.join("subdir");
        fs::create_dir(&sub_dir).unwrap();

        let repo = Repository::discover_from(&sub_dir).unwrap();
        assert_eq!(repo.path(), repo_path.as_path());
    }

    #[test]
    fn test_discover_not_a_repo() {
        let temp_dir = TempDir::new().unwrap();
        let result = Repository::discover_from(temp_dir.path());

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GitError::NotARepository));
    }

    #[test]
    fn test_empty_repo_state() {
        let (_temp, repo_path) = create_test_repo();
        let repo = Repository::new(&repo_path);

        let state = repo.state().unwrap();
        assert!(state.current_branch.is_some());
        assert_eq!(state.current_branch.as_deref(), Some("main"));
        assert!(state.is_clean());
        assert!(!state.is_detached());
        assert_eq!(state.recent_commits.len(), 0);
        assert_eq!(state.stashes.len(), 0);
    }

    #[test]
    fn test_repo_with_files() {
        let (_temp, repo_path) = create_test_repo();
        let repo = Repository::new(&repo_path);

        // Create a file
        let test_file = repo_path.join("test.txt");
        fs::write(&test_file, "test content").unwrap();

        let state = repo.state().unwrap();
        assert!(!state.is_clean());
        assert_eq!(state.untracked_files.len(), 1);
        assert_eq!(state.untracked_files[0].path, "test.txt");
    }

    #[test]
    fn test_repo_with_staged_file() {
        let (_temp, repo_path) = create_test_repo();
        let repo = Repository::new(&repo_path);

        // Create and stage a file
        let test_file = repo_path.join("staged.txt");
        fs::write(&test_file, "staged content").unwrap();

        Command::new("git")
            .args(&["add", "staged.txt"])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        let state = repo.state().unwrap();
        assert!(!state.is_clean());
        assert_eq!(state.staged_files.len(), 1);
    }
}
