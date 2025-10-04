use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Helper to create a test git repository
pub fn create_test_repo() -> (TempDir, PathBuf) {
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
pub fn create_commit(repo_path: &PathBuf, file: &str, content: &str, message: &str) {
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
