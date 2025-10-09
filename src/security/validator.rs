use std::collections::HashSet;
use thiserror::Error;
use crate::security::ALLOWED_GIT_SUBCOMMANDS;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Git subcommand not allowed: {0}")]
    DisallowedSubcommand(String),

    #[error("Command contains suspicious operators: {0}")]
    SuspiciousOperators(String),

    #[error("Command contains dangerous flags: {0}")]
    DangerousFlags(String),

    #[error("Invalid command format")]
    InvalidFormat,

    #[error("Empty command")]
    EmptyCommand,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DangerousOp {
    ForcePush,
    HardReset,
    Clean,
    FilterBranch,
    ForceCheckout,
    DeleteBranch,
    Rebase,
}

#[derive(Debug, Clone)]
pub struct ValidatedCommand {
    pub command: String,
    pub is_dangerous: bool,
    pub danger_type: Option<DangerousOp>,
}

pub struct CommandValidator {
    allowed_subcommands: HashSet<&'static str>,
    dangerous_flags: HashSet<&'static str>,
}

impl CommandValidator {
    pub fn new() -> Self {
        // Use shared allowlist from security module
        let allowed_subcommands = ALLOWED_GIT_SUBCOMMANDS
            .iter()
            .copied()
            .collect();

        let dangerous_flags = ["--exec", "core.sshCommand", "-C"]
            .iter()
            .copied()
            .collect();

        Self {
            allowed_subcommands,
            dangerous_flags,
        }
    }

    /// Validate a git command
    pub fn validate(&self, command: &str) -> Result<ValidatedCommand, ValidationError> {
        let command = command.trim();

        if command.is_empty() {
            return Err(ValidationError::EmptyCommand);
        }

        // Check for command injection attempts
        self.check_for_injection(command)?;

        // Check for dangerous flags BEFORE extracting subcommand
        // (since flags might interfere with subcommand extraction)
        self.check_dangerous_flags(command)?;

        // Extract subcommand
        let subcommand = self.extract_subcommand(command)?;

        // Check against allowlist
        if !self.check_subcommand(subcommand) {
            return Err(ValidationError::DisallowedSubcommand(
                subcommand.to_string(),
            ));
        }

        // Detect dangerous operations
        let danger_type = self.detect_dangerous_ops(command);
        let is_dangerous = danger_type.is_some();

        Ok(ValidatedCommand {
            command: command.to_string(),
            is_dangerous,
            danger_type,
        })
    }

    /// Extract the git subcommand from the command string
    fn extract_subcommand<'a>(&self, command: &'a str) -> Result<&'a str, ValidationError> {
        // Remove "git " prefix if present
        let cmd = command.strip_prefix("git ").unwrap_or(command);

        // Skip flags (words starting with -) to find the actual subcommand
        for word in cmd.split_whitespace() {
            if !word.starts_with('-') {
                return Ok(word);
            }
        }

        Err(ValidationError::InvalidFormat)
    }

    /// Check if subcommand is in allowlist
    fn check_subcommand(&self, subcommand: &str) -> bool {
        self.allowed_subcommands.contains(subcommand)
    }

    /// Check for command injection attempts
    fn check_for_injection(&self, command: &str) -> Result<(), ValidationError> {
        // Check for suspicious operators (excluding &&)
        let suspicious_operators = [";", "||", "|", ">", "<", "$(", "`"];

        for op in &suspicious_operators {
            if command.contains(op) {
                return Err(ValidationError::SuspiciousOperators(op.to_string()));
            }
        }

        // Check for && - only allowed between git commands
        if command.contains("&&") {
            // Verify both sides are git commands
            let parts: Vec<&str> = command.split("&&").collect();
            for part in parts {
                let trimmed = part.trim();
                if !trimmed.starts_with("git ") && !self.is_likely_git_subcommand(trimmed) {
                    return Err(ValidationError::SuspiciousOperators(
                        "&& with non-git command".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Check if a command starts with a git subcommand (for commands without "git " prefix)
    fn is_likely_git_subcommand(&self, cmd: &str) -> bool {
        if let Some(first_word) = cmd.split_whitespace().next() {
            self.allowed_subcommands.contains(first_word)
        } else {
            false
        }
    }

    /// Check for dangerous flags that could enable arbitrary code execution
    fn check_dangerous_flags(&self, command: &str) -> Result<(), ValidationError> {
        // Check for -c flag which can set arbitrary git config
        if command.contains(" -c ") || command.starts_with("-c ") {
            return Err(ValidationError::DangerousFlags("-c".to_string()));
        }

        // Check for -C flag which can run git in arbitrary directories
        if command.contains(" -C ") || command.starts_with("-C ") {
            return Err(ValidationError::DangerousFlags("-C".to_string()));
        }

        // Check for other dangerous flags
        for flag in &self.dangerous_flags {
            if command.contains(flag) {
                return Err(ValidationError::DangerousFlags(flag.to_string()));
            }
        }
        Ok(())
    }

    /// Detect dangerous operations
    fn detect_dangerous_ops(&self, command: &str) -> Option<DangerousOp> {
        let cmd_lower = command.to_lowercase();

        // Force push (must check before other -f flags)
        if cmd_lower.contains("push") && (cmd_lower.contains("--force") || cmd_lower.contains("-f"))
        {
            return Some(DangerousOp::ForcePush);
        }

        // Hard reset
        if cmd_lower.contains("reset") && cmd_lower.contains("--hard") {
            return Some(DangerousOp::HardReset);
        }

        // Clean with force
        if cmd_lower.contains("clean")
            && (cmd_lower.contains("-f") || cmd_lower.contains("--force"))
        {
            return Some(DangerousOp::Clean);
        }

        // Filter-branch
        if cmd_lower.contains("filter-branch") {
            return Some(DangerousOp::FilterBranch);
        }

        // Force checkout
        if cmd_lower.contains("checkout") && (cmd_lower.contains("--force") || cmd_lower.contains("-f")) {
            return Some(DangerousOp::ForceCheckout);
        }

        // Delete branch (-D flag)
        if cmd_lower.contains("branch") && cmd_lower.contains("-d") {
            return Some(DangerousOp::DeleteBranch);
        }

        // Rebase (interactive or not)
        if cmd_lower.contains("rebase") {
            return Some(DangerousOp::Rebase);
        }

        None
    }
}

impl Default for CommandValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_simple_command() {
        let validator = CommandValidator::new();
        let result = validator.validate("git status");
        assert!(result.is_ok());

        let validated = result.unwrap();
        assert_eq!(validated.command, "git status");
        assert!(!validated.is_dangerous);
        assert!(validated.danger_type.is_none());
    }

    #[test]
    fn test_validate_without_git_prefix() {
        let validator = CommandValidator::new();
        let result = validator.validate("status");
        assert!(result.is_ok());
    }

    #[test]
    fn test_disallowed_subcommand() {
        let validator = CommandValidator::new();
        let result = validator.validate("git rm -rf /");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ValidationError::DisallowedSubcommand(_)
        ));
    }

    #[test]
    fn test_semicolon_injection() {
        let validator = CommandValidator::new();
        let result = validator.validate("git status; rm -rf /");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ValidationError::SuspiciousOperators(_)
        ));
    }

    #[test]
    fn test_pipe_injection() {
        let validator = CommandValidator::new();
        let result = validator.validate("git log | sh");
        assert!(result.is_err());
    }

    #[test]
    fn test_redirect_injection() {
        let validator = CommandValidator::new();
        let result = validator.validate("git status > /etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn test_command_substitution() {
        let validator = CommandValidator::new();
        let result = validator.validate("git status $(whoami)");
        assert!(result.is_err());
    }

    #[test]
    fn test_backtick_substitution() {
        let validator = CommandValidator::new();
        let result = validator.validate("git status `whoami`");
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_and_operator() {
        let validator = CommandValidator::new();
        let result = validator.validate("git add -A && git commit -m 'test'");
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_and_operator() {
        let validator = CommandValidator::new();
        let result = validator.validate("git status && rm -rf /");
        assert!(result.is_err());
    }

    #[test]
    fn test_dangerous_flag_exec() {
        let validator = CommandValidator::new();
        let result = validator.validate("git -c core.pager='sh -c whoami' log");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ValidationError::DangerousFlags(_)
        ));
    }

    #[test]
    fn test_dangerous_flag_c_directory() {
        let validator = CommandValidator::new();

        // Test -C with space
        let result = validator.validate("git -C /etc status");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ValidationError::DangerousFlags(_)
        ));

        // Test -C at start
        let result2 = validator.validate("-C /tmp git status");
        assert!(result2.is_err());

        // Test git -C with sensitive path
        let result3 = validator.validate("git -C /root status");
        assert!(result3.is_err());
    }

    #[test]
    fn test_force_push_detection() {
        let validator = CommandValidator::new();
        let result = validator.validate("git push --force origin main");
        assert!(result.is_ok());

        let validated = result.unwrap();
        assert!(validated.is_dangerous);
        assert_eq!(validated.danger_type, Some(DangerousOp::ForcePush));
    }

    #[test]
    fn test_force_push_short_flag() {
        let validator = CommandValidator::new();
        let result = validator.validate("git push -f origin main");
        assert!(result.is_ok());

        let validated = result.unwrap();
        assert!(validated.is_dangerous);
    }

    #[test]
    fn test_hard_reset_detection() {
        let validator = CommandValidator::new();
        let result = validator.validate("git reset --hard HEAD~1");
        assert!(result.is_ok());

        let validated = result.unwrap();
        assert!(validated.is_dangerous);
        assert_eq!(validated.danger_type, Some(DangerousOp::HardReset));
    }

    #[test]
    fn test_clean_detection() {
        let validator = CommandValidator::new();
        let result = validator.validate("git clean -fd");
        assert!(result.is_ok());

        let validated = result.unwrap();
        assert!(validated.is_dangerous);
        assert_eq!(validated.danger_type, Some(DangerousOp::Clean));
    }

    #[test]
    fn test_filter_branch_detection() {
        let validator = CommandValidator::new();
        let result = validator.validate("git filter-branch --tree-filter 'rm file' HEAD");
        assert!(result.is_ok());

        let validated = result.unwrap();
        assert!(validated.is_dangerous);
        assert_eq!(validated.danger_type, Some(DangerousOp::FilterBranch));
    }

    #[test]
    fn test_empty_command() {
        let validator = CommandValidator::new();
        let result = validator.validate("");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ValidationError::EmptyCommand));
    }

    #[test]
    fn test_allowed_subcommands() {
        let validator = CommandValidator::new();

        // Test all allowed subcommands
        let commands = vec![
            "git status",
            "git log",
            "git show",
            "git diff",
            "git branch",
            "git tag",
            "git remote",
            "git reflog",
            "git blame",
            "git describe",
            "git add .",
            "git commit -m 'test'",
            "git checkout main",
            "git switch feature",
            "git restore file.txt",
            "git reset HEAD",
            "git revert abc123",
            "git merge feature",
            "git rebase main",
            "git cherry-pick abc123",
            "git stash",
            "git clean -n",
            "git push origin main",
            "git pull origin main",
            "git fetch origin",
            "git clone repo.git",
            "git config user.name",
        ];

        for cmd in commands {
            let result = validator.validate(cmd);
            assert!(result.is_ok(), "Command should be valid: {}", cmd);
        }
    }

    #[test]
    fn test_force_checkout_detection() {
        let validator = CommandValidator::new();
        let result = validator.validate("git checkout --force main");
        assert!(result.is_ok());

        let validated = result.unwrap();
        assert!(validated.is_dangerous);
        assert_eq!(validated.danger_type, Some(DangerousOp::ForceCheckout));
    }

    #[test]
    fn test_force_checkout_short_flag() {
        let validator = CommandValidator::new();
        let result = validator.validate("git checkout -f main");
        assert!(result.is_ok());

        let validated = result.unwrap();
        assert!(validated.is_dangerous);
        assert_eq!(validated.danger_type, Some(DangerousOp::ForceCheckout));
    }

    #[test]
    fn test_delete_branch_detection() {
        let validator = CommandValidator::new();
        let result = validator.validate("git branch -D feature-branch");
        assert!(result.is_ok());

        let validated = result.unwrap();
        assert!(validated.is_dangerous);
        assert_eq!(validated.danger_type, Some(DangerousOp::DeleteBranch));
    }

    #[test]
    fn test_delete_branch_lowercase() {
        let validator = CommandValidator::new();
        let result = validator.validate("git branch -d feature-branch");
        assert!(result.is_ok());

        let validated = result.unwrap();
        assert!(validated.is_dangerous);
        assert_eq!(validated.danger_type, Some(DangerousOp::DeleteBranch));
    }

    #[test]
    fn test_rebase_detection() {
        let validator = CommandValidator::new();
        let result = validator.validate("git rebase main");
        assert!(result.is_ok());

        let validated = result.unwrap();
        assert!(validated.is_dangerous);
        assert_eq!(validated.danger_type, Some(DangerousOp::Rebase));
    }

    #[test]
    fn test_rebase_interactive_detection() {
        let validator = CommandValidator::new();
        let result = validator.validate("git rebase -i HEAD~3");
        assert!(result.is_ok());

        let validated = result.unwrap();
        assert!(validated.is_dangerous);
        assert_eq!(validated.danger_type, Some(DangerousOp::Rebase));
    }
}
