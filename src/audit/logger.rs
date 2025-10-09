use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use chrono::Utc;

const MAX_LOG_SIZE: u64 = 10 * 1024 * 1024; // 10MB

pub struct AuditLogger {
    log_path: PathBuf,
}

impl AuditLogger {
    /// Create a new AuditLogger with the default log path
    pub fn new() -> std::io::Result<Self> {
        let log_path = Self::default_log_path()?;

        // Ensure directory exists
        if let Some(parent) = log_path.parent() {
            fs::create_dir_all(parent)?;
        }

        Ok(Self { log_path })
    }

    /// Create an AuditLogger with a custom log path
    pub fn with_path<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let log_path = path.as_ref().to_path_buf();

        // Ensure directory exists
        if let Some(parent) = log_path.parent() {
            fs::create_dir_all(parent)?;
        }

        Ok(Self { log_path })
    }

    /// Get the default log path: ~/.config/gitalky/history.log
    fn default_log_path() -> std::io::Result<PathBuf> {
        let home = std::env::var("HOME")
            .map_err(|_| std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "HOME environment variable not set"
            ))?;

        Ok(PathBuf::from(home)
            .join(".config")
            .join("gitalky")
            .join("history.log"))
    }

    /// Log a command execution
    pub fn log_command(
        &self,
        command: &str,
        repo_path: &Path,
        exit_code: i32,
    ) -> std::io::Result<()> {
        // Check and rotate log if needed
        self.rotate_if_needed()?;

        let timestamp = Utc::now().to_rfc3339();
        let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
        let repo_path_str = repo_path.display();

        let log_entry = format!(
            "[{}] [{}] [{}] [exit:{}] {}\n",
            timestamp, user, repo_path_str, exit_code, command
        );

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;

        file.write_all(log_entry.as_bytes())?;
        file.flush()?;

        Ok(())
    }

    /// Log a validation failure for forensics
    ///
    /// Records when LLM output or user input fails validation checks.
    /// This helps detect attack patterns and LLM misbehavior.
    pub fn log_validation_failure(
        &self,
        query: &str,
        llm_output: &str,
        reason: &str,
        repo_path: &Path,
    ) -> std::io::Result<()> {
        // Check and rotate log if needed
        self.rotate_if_needed()?;

        let timestamp = Utc::now().to_rfc3339();
        let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
        let repo_path_str = repo_path.display();

        let log_entry = format!(
            "[{}] [{}] [{}] [VALIDATION-REJECTED] query=\"{}\" llm_output=\"{}\" reason=\"{}\"\n",
            timestamp, user, repo_path_str, query, llm_output, reason
        );

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;

        file.write_all(log_entry.as_bytes())?;
        file.flush()?;

        Ok(())
    }

    /// Rotate log file if it exceeds MAX_LOG_SIZE
    fn rotate_if_needed(&self) -> std::io::Result<()> {
        if !self.log_path.exists() {
            return Ok(());
        }

        let metadata = fs::metadata(&self.log_path)?;
        if metadata.len() > MAX_LOG_SIZE {
            // Rotate: history.log -> history.log.1
            let backup_path = self.log_path.with_extension("log.1");
            fs::rename(&self.log_path, backup_path)?;
        }

        Ok(())
    }

    /// Get the path to the log file
    pub fn log_path(&self) -> &Path {
        &self.log_path
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new().expect("Failed to create default AuditLogger")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_logger() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");

        let logger = AuditLogger::with_path(&log_path).unwrap();
        assert_eq!(logger.log_path(), log_path);
    }

    #[test]
    fn test_log_command() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");

        let logger = AuditLogger::with_path(&log_path).unwrap();
        let repo_path = Path::new("/test/repo");

        logger.log_command("git status", repo_path, 0).unwrap();

        // Verify log file exists
        assert!(log_path.exists());

        // Verify content
        let content = fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("git status"));
        assert!(content.contains("/test/repo"));
        assert!(content.contains("exit:0"));
    }

    #[test]
    fn test_multiple_log_entries() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");

        let logger = AuditLogger::with_path(&log_path).unwrap();
        let repo_path = Path::new("/test/repo");

        logger.log_command("git status", repo_path, 0).unwrap();
        logger.log_command("git add .", repo_path, 0).unwrap();
        logger.log_command("git commit -m 'test'", repo_path, 0).unwrap();

        let content = fs::read_to_string(&log_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(content.contains("git status"));
        assert!(content.contains("git add ."));
        assert!(content.contains("git commit"));
    }

    #[test]
    fn test_log_rotation() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");

        let logger = AuditLogger::with_path(&log_path).unwrap();
        let repo_path = Path::new("/test/repo");

        // Write a large entry to trigger rotation
        let large_command = "git ".to_string() + &"x".repeat(MAX_LOG_SIZE as usize);
        logger.log_command(&large_command, repo_path, 0).unwrap();

        // Write another entry - should trigger rotation
        logger.log_command("git status", repo_path, 0).unwrap();

        // Check backup file exists
        let backup_path = log_path.with_extension("log.1");
        assert!(backup_path.exists());

        // New log should exist and be smaller
        assert!(log_path.exists());
        let metadata = fs::metadata(&log_path).unwrap();
        assert!(metadata.len() < MAX_LOG_SIZE);
    }

    #[test]
    fn test_log_with_failed_command() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");

        let logger = AuditLogger::with_path(&log_path).unwrap();
        let repo_path = Path::new("/test/repo");

        logger.log_command("git invalid-command", repo_path, 128).unwrap();

        let content = fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("exit:128"));
        assert!(content.contains("git invalid-command"));
    }

    #[test]
    fn test_log_validation_failure() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");

        let logger = AuditLogger::with_path(&log_path).unwrap();
        let repo_path = Path::new("/test/repo");

        logger
            .log_validation_failure(
                "show me the status",
                "rm -rf /",
                "LLM output doesn't look like a git command",
                repo_path,
            )
            .unwrap();

        let content = fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("VALIDATION-REJECTED"));
        assert!(content.contains("show me the status"));
        assert!(content.contains("rm -rf /"));
        assert!(content.contains("doesn't look like a git command"));
    }

    #[test]
    fn test_log_validation_failure_shell_injection() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");

        let logger = AuditLogger::with_path(&log_path).unwrap();
        let repo_path = Path::new("/test/repo");

        logger
            .log_validation_failure(
                "check status",
                "git status; rm -rf /",
                "LLM output contains shell metacharacter ';'",
                repo_path,
            )
            .unwrap();

        let content = fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("VALIDATION-REJECTED"));
        assert!(content.contains("git status; rm -rf /"));
        assert!(content.contains("shell metacharacter"));
    }
}
