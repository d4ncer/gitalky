use crate::error::{GitError, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::Duration;

/// Result of executing a git command
#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub success: bool,
}

/// Executes git commands within a repository
#[derive(Debug, Clone)]
pub struct GitExecutor {
    repo_path: PathBuf,
}

impl GitExecutor {
    /// Create a new GitExecutor for the given repository path
    pub fn new<P: AsRef<Path>>(repo_path: P) -> Self {
        Self {
            repo_path: repo_path.as_ref().to_path_buf(),
        }
    }

    /// Execute a git command and return the output
    ///
    /// The command string should not include "git" prefix
    /// Example: executor.execute("status --porcelain")
    pub fn execute(&self, command: &str) -> Result<CommandOutput> {
        self.execute_with_timeout(command, Duration::from_secs(30))
    }

    /// Execute a git command with a custom timeout
    pub fn execute_with_timeout(&self, command: &str, _timeout: Duration) -> Result<CommandOutput> {
        // Basic input sanitization - no shell interpolation
        if command.contains('$') || command.contains('`') {
            return Err(GitError::CommandFailed(
                "Command contains potentially unsafe characters".to_string(),
            ));
        }

        // Split command into args, respecting quotes
        let args = self.parse_command(command)?;
        if args.is_empty() {
            return Err(GitError::CommandFailed("Empty command".to_string()));
        }

        // Execute git command
        let output = Command::new("git")
            .args(&args)
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| GitError::CommandFailed(format!("Failed to execute git: {}", e)))?;

        self.process_output(output, command)
    }

    /// Parse command string respecting single and double quotes
    fn parse_command(&self, command: &str) -> Result<Vec<String>> {
        let mut args = Vec::new();
        let mut current_arg = String::new();
        let mut in_single_quote = false;
        let mut in_double_quote = false;

        for ch in command.chars() {
            match ch {
                '\'' if !in_double_quote => {
                    in_single_quote = !in_single_quote;
                }
                '"' if !in_single_quote => {
                    in_double_quote = !in_double_quote;
                }
                ' ' | '\t' if !in_single_quote && !in_double_quote => {
                    if !current_arg.is_empty() {
                        args.push(current_arg.clone());
                        current_arg.clear();
                    }
                }
                _ => {
                    current_arg.push(ch);
                }
            }
        }

        // Push final argument if any
        if !current_arg.is_empty() {
            args.push(current_arg);
        }

        // Check for unclosed quotes
        if in_single_quote || in_double_quote {
            return Err(GitError::CommandFailed(
                "Unclosed quote in command".to_string(),
            ));
        }

        Ok(args)
    }

    /// Process command output into CommandOutput struct
    fn process_output(&self, output: Output, command: &str) -> Result<CommandOutput> {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);
        let success = output.status.success();

        let cmd_output = CommandOutput {
            stdout,
            stderr: stderr.clone(),
            exit_code,
            success,
        };

        // Return error for failed commands
        if !success {
            return Err(GitError::CommandFailed(format!(
                "Command 'git {}' failed with exit code {}: {}",
                command,
                exit_code,
                stderr.trim()
            )));
        }

        Ok(cmd_output)
    }

    /// Get the repository path
    pub fn repo_path(&self) -> &Path {
        &self.repo_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_repo() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().to_path_buf();

        // Initialize git repo
        Command::new("git")
            .args(["init"])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        // Configure git
        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(&repo_path)
            .output()
            .unwrap();

        (temp_dir, repo_path)
    }

    #[test]
    fn test_execute_status() {
        let (_temp, repo_path) = create_test_repo();
        let executor = GitExecutor::new(&repo_path);

        let result = executor.execute("status --porcelain");
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert_eq!(output.exit_code, 0);
    }

    #[test]
    fn test_execute_log_empty_repo() {
        let (_temp, repo_path) = create_test_repo();
        let executor = GitExecutor::new(&repo_path);

        // Log command should fail in empty repo
        let result = executor.execute("log --oneline");
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitization_dollar_sign() {
        let (_temp, repo_path) = create_test_repo();
        let executor = GitExecutor::new(&repo_path);

        let result = executor.execute("status $(whoami)");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GitError::CommandFailed(_)));
    }

    #[test]
    fn test_sanitization_backtick() {
        let (_temp, repo_path) = create_test_repo();
        let executor = GitExecutor::new(&repo_path);

        let result = executor.execute("status `whoami`");
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_command() {
        let (_temp, repo_path) = create_test_repo();
        let executor = GitExecutor::new(&repo_path);

        let result = executor.execute("");
        assert!(result.is_err());
    }

    #[test]
    fn test_repo_path() {
        let (_temp, repo_path) = create_test_repo();
        let executor = GitExecutor::new(&repo_path);

        assert_eq!(executor.repo_path(), repo_path.as_path());
    }

    #[test]
    fn test_parse_command_simple() {
        let (_temp, repo_path) = create_test_repo();
        let executor = GitExecutor::new(&repo_path);

        let args = executor.parse_command("status --porcelain").unwrap();
        assert_eq!(args, vec!["status", "--porcelain"]);
    }

    #[test]
    fn test_parse_command_single_quotes() {
        let (_temp, repo_path) = create_test_repo();
        let executor = GitExecutor::new(&repo_path);

        let args = executor
            .parse_command("log --pretty=format:'%h %s' --oneline")
            .unwrap();
        assert_eq!(args, vec!["log", "--pretty=format:%h %s", "--oneline"]);
    }

    #[test]
    fn test_parse_command_double_quotes() {
        let (_temp, repo_path) = create_test_repo();
        let executor = GitExecutor::new(&repo_path);

        let args = executor
            .parse_command(r#"log --pretty=format:"%h %s" --oneline"#)
            .unwrap();
        assert_eq!(args, vec!["log", "--pretty=format:%h %s", "--oneline"]);
    }

    #[test]
    fn test_parse_command_nested_quotes() {
        let (_temp, repo_path) = create_test_repo();
        let executor = GitExecutor::new(&repo_path);

        let args = executor
            .parse_command(r#"commit -m "It's working""#)
            .unwrap();
        assert_eq!(args, vec!["commit", "-m", "It's working"]);
    }

    #[test]
    fn test_parse_command_unclosed_single_quote() {
        let (_temp, repo_path) = create_test_repo();
        let executor = GitExecutor::new(&repo_path);

        let result = executor.parse_command("log --pretty=format:'%h %s");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_command_unclosed_double_quote() {
        let (_temp, repo_path) = create_test_repo();
        let executor = GitExecutor::new(&repo_path);

        let result = executor.parse_command(r#"log --pretty=format:"%h %s"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_command_complex_format() {
        let (_temp, repo_path) = create_test_repo();
        let executor = GitExecutor::new(&repo_path);

        let args = executor
            .parse_command("log --pretty=format:'%C(yellow)%h%Creset %C(blue)%ad%Creset %C(green)%an%Creset %s' --date=short --graph")
            .unwrap();
        assert_eq!(
            args,
            vec![
                "log",
                "--pretty=format:%C(yellow)%h%Creset %C(blue)%ad%Creset %C(green)%an%Creset %s",
                "--date=short",
                "--graph"
            ]
        );
    }
}
