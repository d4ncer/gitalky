use gitalky::{GitError, GitVersion, Repository};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Helper to create a test git repository
fn create_test_repo() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_path_buf();

    // Initialize git repo
    Command::new("git")
        .args(&["init"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to init git repo");

    // Configure git
    Command::new("git")
        .args(&["config", "user.name", "Test User"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to set git user.name");

    Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to set git user.email");

    (temp_dir, repo_path)
}

/// Helper to create a commit
fn create_commit(repo_path: &PathBuf, file: &str, content: &str, message: &str) {
    let file_path = repo_path.join(file);
    fs::write(&file_path, content).expect("Failed to write file");

    Command::new("git")
        .args(&["add", file])
        .current_dir(repo_path)
        .output()
        .expect("Failed to add file");

    Command::new("git")
        .args(&["commit", "-m", message])
        .current_dir(repo_path)
        .output()
        .expect("Failed to commit");
}

#[test]
fn test_git_version_detection() {
    let version = GitVersion::detect().expect("Failed to detect git version");
    assert!(version.major >= 2);
}

#[test]
fn test_git_version_validation() {
    let version = GitVersion::validate().expect("Git version should be >= 2.20");
    assert!(version.is_supported());
}

#[test]
fn test_discover_repository() {
    let (_temp, repo_path) = create_test_repo();

    let repo = Repository::discover_from(&repo_path).expect("Failed to discover repository");
    assert_eq!(repo.path(), repo_path.as_path());
}

#[test]
fn test_discover_from_subdirectory() {
    let (_temp, repo_path) = create_test_repo();

    // Create subdirectory
    let sub_dir = repo_path.join("subdir");
    fs::create_dir(&sub_dir).expect("Failed to create subdirectory");

    let repo = Repository::discover_from(&sub_dir).expect("Failed to discover from subdirectory");
    assert_eq!(repo.path(), repo_path.as_path());
}

#[test]
fn test_discover_not_a_repository() {
    let temp_dir = TempDir::new().unwrap();
    let result = Repository::discover_from(temp_dir.path());

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), GitError::NotARepository));
}

#[test]
fn test_empty_repository_state() {
    let (_temp, repo_path) = create_test_repo();
    let repo = Repository::new(&repo_path);

    let state = repo.state().expect("Failed to get state");

    assert!(state.current_branch.is_some());
    assert!(state.is_clean());
    assert!(!state.is_detached());
    assert_eq!(state.recent_commits.len(), 0);
    assert_eq!(state.stashes.len(), 0);
    assert!(!state.in_merge);
    assert!(!state.in_rebase);
}

#[test]
fn test_untracked_files() {
    let (_temp, repo_path) = create_test_repo();
    let repo = Repository::new(&repo_path);

    // Create untracked file
    fs::write(repo_path.join("untracked.txt"), "content").expect("Failed to write file");

    let state = repo.state().expect("Failed to get state");

    assert!(!state.is_clean());
    assert_eq!(state.untracked_files.len(), 1);
    assert_eq!(state.untracked_files[0].path, "untracked.txt");
    assert_eq!(state.staged_files.len(), 0);
    assert_eq!(state.unstaged_files.len(), 0);
}

#[test]
fn test_staged_files() {
    let (_temp, repo_path) = create_test_repo();
    let repo = Repository::new(&repo_path);

    // Create and stage file
    fs::write(repo_path.join("staged.txt"), "staged content").expect("Failed to write file");

    Command::new("git")
        .args(&["add", "staged.txt"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to stage file");

    let state = repo.state().expect("Failed to get state");

    assert!(!state.is_clean());
    assert_eq!(state.staged_files.len(), 1);
    assert_eq!(state.staged_files[0].path, "staged.txt");
    assert!(state.staged_files[0].staged);
}

#[test]
fn test_unstaged_modifications() {
    let (_temp, repo_path) = create_test_repo();
    let repo = Repository::new(&repo_path);

    // Create initial commit
    create_commit(&repo_path, "file.txt", "original", "Initial commit");

    // Modify file without staging
    fs::write(repo_path.join("file.txt"), "modified").expect("Failed to modify file");

    let state = repo.state().expect("Failed to get state");

    assert!(!state.is_clean());
    assert_eq!(state.unstaged_files.len(), 1);
    assert_eq!(state.unstaged_files[0].path, "file.txt");
    assert!(state.unstaged_files[0].unstaged);
}

#[test]
fn test_recent_commits() {
    let (_temp, repo_path) = create_test_repo();
    let repo = Repository::new(&repo_path);

    // Create multiple commits
    create_commit(&repo_path, "file1.txt", "content1", "First commit");
    create_commit(&repo_path, "file2.txt", "content2", "Second commit");
    create_commit(&repo_path, "file3.txt", "content3", "Third commit");

    let state = repo.state().expect("Failed to get state");

    assert_eq!(state.recent_commits.len(), 3);
    assert_eq!(state.recent_commits[0].message, "Third commit");
    assert_eq!(state.recent_commits[1].message, "Second commit");
    assert_eq!(state.recent_commits[2].message, "First commit");
}

#[test]
fn test_stash_operations() {
    let (_temp, repo_path) = create_test_repo();
    let repo = Repository::new(&repo_path);

    // Create initial commit
    create_commit(&repo_path, "file.txt", "original", "Initial commit");

    // Modify and stash
    fs::write(repo_path.join("file.txt"), "modified").expect("Failed to modify file");

    Command::new("git")
        .args(&["stash", "push", "-m", "WIP: test stash"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to stash");

    let state = repo.state().expect("Failed to get state");

    assert_eq!(state.stashes.len(), 1);
    assert!(state.stashes[0].message.contains("WIP: test stash"));
    assert!(state.is_clean()); // After stash, working directory is clean
}

#[test]
fn test_detached_head_state() {
    let (_temp, repo_path) = create_test_repo();
    let repo = Repository::new(&repo_path);

    // Create commits
    create_commit(&repo_path, "file1.txt", "content1", "First commit");
    create_commit(&repo_path, "file2.txt", "content2", "Second commit");

    // Get first commit hash
    let output = Command::new("git")
        .args(&["log", "--format=%H", "--reverse"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to get log");

    let first_commit = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap()
        .trim()
        .to_string();

    // Checkout first commit (detached HEAD)
    Command::new("git")
        .args(&["checkout", &first_commit])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to checkout commit");

    let state = repo.state().expect("Failed to get state");

    assert!(state.is_detached());
    assert!(state.current_branch.is_none());
}

#[test]
fn test_merge_in_progress_detection() {
    let (_temp, repo_path) = create_test_repo();
    let repo = Repository::new(&repo_path);

    // Create initial commit
    create_commit(&repo_path, "file.txt", "main content", "Initial commit");

    // Create a branch
    Command::new("git")
        .args(&["checkout", "-b", "feature"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to create branch");

    // Modify on branch
    create_commit(&repo_path, "file.txt", "feature content", "Feature commit");

    // Go back to main
    Command::new("git")
        .args(&["checkout", "main"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to checkout main");

    // Modify on main (create conflict)
    create_commit(&repo_path, "file.txt", "main content 2", "Main commit");

    // Attempt merge (will fail with conflict)
    let _ = Command::new("git")
        .args(&["merge", "feature"])
        .current_dir(&repo_path)
        .output();

    let state = repo.state().expect("Failed to get state");

    assert!(state.in_merge);
}

#[test]
fn test_executor_command_sanitization() {
    let (_temp, repo_path) = create_test_repo();
    let repo = Repository::new(&repo_path);
    let executor = repo.executor();

    // Test that dangerous commands are rejected
    let result = executor.execute("status $(whoami)");
    assert!(result.is_err());

    let result = executor.execute("status `whoami`");
    assert!(result.is_err());
}

#[test]
fn test_mixed_staged_and_unstaged() {
    let (_temp, repo_path) = create_test_repo();
    let repo = Repository::new(&repo_path);

    // Create initial commit
    create_commit(&repo_path, "file.txt", "original", "Initial commit");

    // Modify and stage
    fs::write(repo_path.join("file.txt"), "staged version").expect("Failed to write file");
    Command::new("git")
        .args(&["add", "file.txt"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to stage");

    // Modify again (now unstaged)
    fs::write(repo_path.join("file.txt"), "unstaged version").expect("Failed to write file");

    let state = repo.state().expect("Failed to get state");

    // File should appear in both staged and unstaged
    assert_eq!(state.staged_files.len(), 1);
    assert_eq!(state.unstaged_files.len(), 1);
    assert_eq!(state.staged_files[0].path, "file.txt");
    assert_eq!(state.unstaged_files[0].path, "file.txt");
}
