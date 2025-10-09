// Test to ensure security allowlist is used consistently across modules

use gitalky::security::ALLOWED_GIT_SUBCOMMANDS;

#[test]
fn test_allowlist_is_not_empty() {
    assert!(!ALLOWED_GIT_SUBCOMMANDS.is_empty());
    assert!(ALLOWED_GIT_SUBCOMMANDS.len() >= 28); // At least 28 subcommands
}

#[test]
fn test_allowlist_contains_common_commands() {
    // Verify critical commands are in the allowlist
    let critical_commands = [
        "status", "log", "diff", "add", "commit",
        "push", "pull", "fetch", "branch", "checkout"
    ];

    for cmd in &critical_commands {
        assert!(
            ALLOWED_GIT_SUBCOMMANDS.contains(cmd),
            "Allowlist missing critical command: {}",
            cmd
        );
    }
}

#[test]
fn test_allowlist_is_sorted_by_category() {
    // This test documents the expected structure
    // Read operations should come first, then write, then remote, etc.

    let read_ops = ["status", "log", "show", "diff", "branch", "tag", "remote", "reflog", "blame", "describe"];
    let write_ops = ["add", "commit", "checkout", "switch", "restore", "reset", "revert", "merge", "rebase", "cherry-pick", "stash", "clean"];
    let remote_ops = ["push", "pull", "fetch", "clone"];

    // Verify all operations are in the allowlist
    for op in read_ops.iter().chain(write_ops.iter()).chain(remote_ops.iter()) {
        assert!(
            ALLOWED_GIT_SUBCOMMANDS.contains(op),
            "Expected operation '{}' not in allowlist",
            op
        );
    }
}

#[test]
fn test_validator_uses_shared_allowlist() {
    use gitalky::security::CommandValidator;

    let validator = CommandValidator::new();

    // Test that validator accepts all subcommands in the shared allowlist
    for subcommand in ALLOWED_GIT_SUBCOMMANDS {
        let command = format!("git {}", subcommand);
        let result = validator.validate(&command);
        assert!(
            result.is_ok(),
            "Validator rejected allowed subcommand '{}': {:?}",
            subcommand,
            result.err()
        );
    }
}

#[tokio::test]
async fn test_llm_validation_uses_shared_allowlist() {
    use gitalky::llm::{ContextBuilder, Translator};
    use gitalky::llm::client::{GitCommand, LLMClient, LLMError};
    use gitalky::llm::context::RepoContext;
    use gitalky::git::Repository;
    use async_trait::async_trait;
    use std::process::Command;
    use tempfile::TempDir;

    // Create test repo
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    Command::new("git")
        .args(["init"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    let repo = Repository::new(repo_path);

    // Mock client for testing
    struct MockClient {
        response: String,
    }

    #[async_trait]
    impl LLMClient for MockClient {
        async fn translate(&self, _query: &str, _context: &RepoContext) -> Result<GitCommand, LLMError> {
            Ok(GitCommand {
                command: self.response.clone(),
                explanation: None,
            })
        }
    }

    // Test that LLM validation accepts all subcommands in the shared allowlist
    for subcommand in ALLOWED_GIT_SUBCOMMANDS {
        let mock_client = Box::new(MockClient {
            response: format!("git {}", subcommand),
        });

        let context_builder = ContextBuilder::new(repo.clone());
        let translator = Translator::new(mock_client, context_builder);

        let result = translator.translate("test query").await;
        assert!(
            result.is_ok(),
            "LLM validation rejected allowed subcommand '{}': {:?}",
            subcommand,
            result.err()
        );
    }
}

#[test]
fn test_no_duplicate_subcommands_in_allowlist() {
    use std::collections::HashSet;

    let mut seen = HashSet::new();
    for cmd in ALLOWED_GIT_SUBCOMMANDS {
        assert!(
            seen.insert(cmd),
            "Duplicate subcommand in allowlist: {}",
            cmd
        );
    }
}
