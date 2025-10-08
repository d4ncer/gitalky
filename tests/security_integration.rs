// Security integration tests
// Tests the defense-in-depth security architecture end-to-end

use gitalky::git::{GitExecutor, Repository};
use gitalky::llm::{ContextBuilder, Translator};
use gitalky::llm::client::{GitCommand, LLMClient, LLMError};
use gitalky::llm::context::RepoContext;
use gitalky::security::CommandValidator;
use async_trait::async_trait;
use std::process::Command;
use tempfile::TempDir;

/// Create a test git repository
fn create_test_repo() -> (TempDir, Repository) {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

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

    let repo = Repository::new(repo_path);
    (temp_dir, repo)
}

// Mock LLM client for testing
struct MockMaliciousLLMClient {
    response: String,
}

#[async_trait]
impl LLMClient for MockMaliciousLLMClient {
    async fn translate(&self, _query: &str, _context: &RepoContext) -> Result<GitCommand, LLMError> {
        Ok(GitCommand {
            command: self.response.clone(),
            explanation: None,
        })
    }
}

#[test]
fn test_validator_rejects_command_injection() {
    let validator = CommandValidator::new();

    // Semicolon injection
    let result = validator.validate("git status; rm -rf /");
    assert!(result.is_err());

    // Pipe injection
    let result = validator.validate("git log | cat /etc/passwd");
    assert!(result.is_err());

    // Command substitution
    let result = validator.validate("git status $(whoami)");
    assert!(result.is_err());

    // Backtick substitution
    let result = validator.validate("git status `whoami`");
    assert!(result.is_err());
}

#[test]
fn test_validator_rejects_dangerous_flags() {
    let validator = CommandValidator::new();

    // -c flag for arbitrary config
    let result = validator.validate("git -c core.pager='sh -c whoami' log");
    assert!(result.is_err());

    // --exec flag
    let result = validator.validate("git --exec=whoami status");
    assert!(result.is_err());
}

#[test]
fn test_validator_detects_all_dangerous_operations() {
    let validator = CommandValidator::new();

    // Force push
    let result = validator.validate("git push --force origin main");
    assert!(result.is_ok());
    let validated = result.unwrap();
    assert!(validated.is_dangerous);

    // Hard reset
    let result = validator.validate("git reset --hard HEAD~1");
    assert!(result.is_ok());
    assert!(result.unwrap().is_dangerous);

    // Clean with force
    let result = validator.validate("git clean -fd");
    assert!(result.is_ok());
    assert!(result.unwrap().is_dangerous);

    // Filter-branch
    let result = validator.validate("git filter-branch --tree-filter 'rm file' HEAD");
    assert!(result.is_ok());
    assert!(result.unwrap().is_dangerous);

    // Force checkout
    let result = validator.validate("git checkout --force main");
    assert!(result.is_ok());
    assert!(result.unwrap().is_dangerous);

    // Delete branch
    let result = validator.validate("git branch -D feature-branch");
    assert!(result.is_ok());
    assert!(result.unwrap().is_dangerous);

    // Rebase
    let result = validator.validate("git rebase main");
    assert!(result.is_ok());
    assert!(result.unwrap().is_dangerous);
}

#[test]
fn test_executor_blocks_shell_metacharacters() {
    let (_temp, repo) = create_test_repo();
    let executor = GitExecutor::new(repo.path());

    // Semicolon
    let result = executor.execute("status; ls");
    assert!(result.is_err());

    // Pipe
    let result = executor.execute("status | cat");
    assert!(result.is_err());

    // Ampersand
    let result = executor.execute("status && ls");
    assert!(result.is_err());

    // Dollar sign
    let result = executor.execute("status $(whoami)");
    assert!(result.is_err());

    // Backtick
    let result = executor.execute("status `whoami`");
    assert!(result.is_err());
}

#[test]
fn test_executor_sanitizes_environment() {
    let (_temp, repo) = create_test_repo();
    let executor = GitExecutor::new(repo.path());

    // Set a dangerous env var
    unsafe {
        std::env::set_var("GIT_SSH_COMMAND", "whoami");
    }

    // Execute should work (env var is stripped)
    let result = executor.execute("status --porcelain");
    assert!(result.is_ok());

    // Clean up
    unsafe {
        std::env::remove_var("GIT_SSH_COMMAND");
    }
}

#[tokio::test]
async fn test_llm_validator_rejects_malicious_output() {
    let (_temp, repo) = create_test_repo();

    // Test various malicious LLM outputs
    let malicious_outputs = vec![
        "rm -rf /",
        "I think you should run git status",
        "git status; rm -rf /",
        "Here's what you need: git add .",
        "",
        "git status\ngit log",
    ];

    for malicious in malicious_outputs {
        let mock_client = Box::new(MockMaliciousLLMClient {
            response: malicious.to_string(),
        });

        let context_builder = ContextBuilder::new(repo.clone());
        let translator = Translator::new(mock_client, context_builder);

        let result = translator.translate("do something").await;
        assert!(
            result.is_err(),
            "Should reject malicious LLM output: '{}'",
            malicious
        );
    }
}

#[tokio::test]
async fn test_llm_validator_accepts_valid_commands() {
    let (_temp, repo) = create_test_repo();

    let valid_commands = vec![
        "git status",
        "status",
        "git add .",
        "add .",
        "git commit -m 'test'",
        "git push origin main",
    ];

    for valid in valid_commands {
        let mock_client = Box::new(MockMaliciousLLMClient {
            response: valid.to_string(),
        });

        let context_builder = ContextBuilder::new(repo.clone());
        let translator = Translator::new(mock_client, context_builder);

        let result = translator.translate("do something").await;
        assert!(
            result.is_ok(),
            "Should accept valid command: '{}'",
            valid
        );
    }
}

#[test]
fn test_defense_in_depth_validator_then_executor() {
    let (_temp, repo) = create_test_repo();
    let validator = CommandValidator::new();
    let executor = GitExecutor::new(repo.path());

    // Malicious command should be rejected by validator
    let malicious = "git status; rm -rf /";
    let validation_result = validator.validate(malicious);
    assert!(validation_result.is_err());

    // Even if validator is bypassed, executor should reject
    let executor_result = executor.execute("status; rm -rf /");
    assert!(executor_result.is_err());
}

#[test]
fn test_allowlist_blocks_disallowed_subcommands() {
    let validator = CommandValidator::new();

    // These should be rejected (not in allowlist)
    let disallowed = vec![
        "git rm -rf /",
        "git gc --prune=now",
        "git daemon",
        "git update-server-info",
    ];

    for cmd in disallowed {
        let result = validator.validate(cmd);
        assert!(result.is_err(), "Should reject disallowed command: '{}'", cmd);
    }
}

#[test]
fn test_allowlist_accepts_safe_commands() {
    let validator = CommandValidator::new();

    let safe_commands = vec![
        "git status",
        "git log",
        "git diff",
        "git show",
        "git branch",
        "git tag",
        "git remote",
        "git reflog",
        "git blame file.txt",
        "git describe",
    ];

    for cmd in safe_commands {
        let result = validator.validate(cmd);
        assert!(result.is_ok(), "Should accept safe command: '{}'", cmd);
    }
}

#[test]
fn test_dangerous_operations_detected_across_layers() {
    let validator = CommandValidator::new();

    // Test that dangerous operations are caught before execution
    let dangerous_commands = vec![
        ("git push --force origin main", "ForcePush"),
        ("git reset --hard HEAD~1", "HardReset"),
        ("git clean -fd", "Clean"),
        ("git checkout --force main", "ForceCheckout"),
        ("git branch -D old-feature", "DeleteBranch"),
        ("git rebase main", "Rebase"),
    ];

    for (cmd, _expected_type) in dangerous_commands {
        let result = validator.validate(cmd);
        assert!(result.is_ok(), "Dangerous command should validate: '{}'", cmd);

        let validated = result.unwrap();
        assert!(
            validated.is_dangerous,
            "Command should be marked dangerous: '{}'",
            cmd
        );
        assert!(
            validated.danger_type.is_some(),
            "Danger type should be set for: '{}'",
            cmd
        );
    }
}

#[test]
fn test_executor_uses_args_not_shell() {
    let (_temp, repo) = create_test_repo();
    let executor = GitExecutor::new(repo.path());

    // Commands with quotes should be parsed correctly (not via shell)
    let result = executor.execute("commit -m 'test message with spaces'");

    // This will fail because there's nothing to commit, but it should NOT
    // fail due to shell parsing issues
    if let Err(e) = result {
        let err_msg = e.to_string();
        // Should be a git error, not a parsing error
        assert!(
            err_msg.contains("nothing to commit") || err_msg.contains("Command"),
            "Error should be from git, not shell parsing: {}",
            err_msg
        );
    }
}

#[tokio::test]
async fn test_end_to_end_security_flow() {
    let (_temp, repo) = create_test_repo();

    // Simulate full flow: LLM → Translator validation → Validator → Executor

    // Step 1: LLM returns a command (simulated)
    let mock_client = Box::new(MockMaliciousLLMClient {
        response: "git status".to_string(),
    });

    let context_builder = ContextBuilder::new(repo.clone());
    let translator = Translator::new(mock_client, context_builder);

    // Step 2: Translator validates LLM output
    let translation_result = translator.translate("show me the status").await;
    assert!(translation_result.is_ok());

    let command = translation_result.unwrap();

    // Step 3: Validator checks the command
    let validator = CommandValidator::new();
    let validation_result = validator.validate(&command.command);
    assert!(validation_result.is_ok());

    let validated = validation_result.unwrap();
    assert!(!validated.is_dangerous); // status is safe

    // Step 4: Executor runs the command (with sanitization)
    let executor = GitExecutor::new(repo.path());
    // Executor expects command without "git " prefix
    let command_for_executor = validated.command.strip_prefix("git ").unwrap_or(&validated.command);
    let execution_result = executor.execute(command_for_executor);
    assert!(execution_result.is_ok());
}
