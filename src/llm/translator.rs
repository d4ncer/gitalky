use crate::llm::client::{GitCommand, LLMClient, LLMError};
use crate::llm::context::ContextBuilder;
use crate::security::ALLOWED_GIT_SUBCOMMANDS;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TranslationError {
    #[error("LLM error: {0}")]
    LLMError(#[from] LLMError),

    #[error("Context building error: {0}")]
    ContextError(#[from] crate::error::GitError),

    #[error("LLM returned invalid output: {0}")]
    InvalidOutput(String),
}

pub struct Translator {
    client: Box<dyn LLMClient>,
    context_builder: ContextBuilder,
}

impl Translator {
    pub fn new(client: Box<dyn LLMClient>, context_builder: ContextBuilder) -> Self {
        Self {
            client,
            context_builder,
        }
    }

    pub async fn translate(&self, query: &str) -> Result<GitCommand, TranslationError> {
        // Classify the query to determine context needs
        let query_type = ContextBuilder::classify_query(query);

        // Build appropriate context
        let context = self.context_builder.build_escalated_context(query_type)?;

        // Translate using LLM
        let command = self.client.translate(query, &context).await?;

        // Validate LLM output before returning
        Self::validate_llm_output(&command.command)?;

        Ok(command)
    }

    /// Validate that LLM output looks like a git command
    fn validate_llm_output(output: &str) -> Result<(), TranslationError> {
        let trimmed = output.trim();

        // Check for empty output
        if trimmed.is_empty() {
            return Err(TranslationError::InvalidOutput(
                "LLM returned empty command".to_string(),
            ));
        }

        // Check for excessively long output (likely hallucination/explanation)
        if trimmed.len() > 500 {
            return Err(TranslationError::InvalidOutput(
                format!("LLM output too long ({} chars), expected git command", trimmed.len()),
            ));
        }

        // Check if it contains newlines (likely explanation, not a command)
        if trimmed.contains('\n') {
            return Err(TranslationError::InvalidOutput(
                "LLM output contains newlines, expected single git command".to_string(),
            ));
        }

        // Check for shell metacharacters (command injection attempts)
        let shell_metacharacters = [";", "|", "&", "$", "`", ">", "<"];
        for meta in &shell_metacharacters {
            if trimmed.contains(meta) {
                return Err(TranslationError::InvalidOutput(
                    format!("LLM output contains shell metacharacter '{}': '{}'", meta, trimmed),
                ));
            }
        }

        // Check if it starts with "git " or looks like a git subcommand
        let starts_with_git = trimmed.starts_with("git ");
        let first_word = trimmed.split_whitespace().next().unwrap_or("");

        // Use shared allowlist from security module (same as validator)
        let looks_like_git = starts_with_git || ALLOWED_GIT_SUBCOMMANDS.contains(&first_word);

        if !looks_like_git {
            return Err(TranslationError::InvalidOutput(
                format!("LLM output doesn't look like a git command: '{}'", trimmed),
            ));
        }

        // Check for suspicious content that might indicate hallucination
        let suspicious_patterns = [
            "I think", "I would", "You should", "Please", "Here's",
            "Let me", "To do this", "First,", "Then,", "Finally,",
        ];

        for pattern in &suspicious_patterns {
            if trimmed.contains(pattern) {
                return Err(TranslationError::InvalidOutput(
                    format!("LLM output contains explanation text: '{}'", trimmed),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::client::LLMError;
    use crate::llm::context::RepoContext;
    use async_trait::async_trait;

    struct MockLLMClient {
        response: String,
    }

    #[async_trait]
    impl LLMClient for MockLLMClient {
        async fn translate(&self, _query: &str, _context: &RepoContext) -> Result<GitCommand, LLMError> {
            Ok(GitCommand {
                command: self.response.clone(),
                explanation: None,
            })
        }
    }

    #[tokio::test]
    async fn test_translator_basic() {
        use crate::git::Repository;

        // This test requires a real git repo
        if let Ok(repo) = Repository::discover() {
            let mock_client = Box::new(MockLLMClient {
                response: "git status".to_string(),
            });

            let context_builder = ContextBuilder::new(repo);
            let translator = Translator::new(mock_client, context_builder);

            let result = translator.translate("show me the status").await;
            assert!(result.is_ok());

            let command = result.unwrap();
            assert_eq!(command.command, "git status");
        }
    }

    // LLM output validation tests
    #[test]
    fn test_validate_llm_output_valid_with_git_prefix() {
        assert!(Translator::validate_llm_output("git status").is_ok());
        assert!(Translator::validate_llm_output("git add .").is_ok());
        assert!(Translator::validate_llm_output("git commit -m 'test'").is_ok());
        assert!(Translator::validate_llm_output("git push origin main").is_ok());
    }

    #[test]
    fn test_validate_llm_output_valid_without_git_prefix() {
        assert!(Translator::validate_llm_output("status").is_ok());
        assert!(Translator::validate_llm_output("add .").is_ok());
        assert!(Translator::validate_llm_output("commit -m 'test'").is_ok());
        assert!(Translator::validate_llm_output("push origin main").is_ok());
    }

    #[test]
    fn test_validate_llm_output_empty() {
        let result = Translator::validate_llm_output("");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TranslationError::InvalidOutput(_)));
    }

    #[test]
    fn test_validate_llm_output_whitespace_only() {
        let result = Translator::validate_llm_output("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_llm_output_too_long() {
        let long_string = "git ".to_string() + &"a".repeat(500);
        let result = Translator::validate_llm_output(&long_string);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TranslationError::InvalidOutput(_)));
    }

    #[test]
    fn test_validate_llm_output_contains_newlines() {
        let result = Translator::validate_llm_output("git status\ngit log");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TranslationError::InvalidOutput(_)));
    }

    #[test]
    fn test_validate_llm_output_contains_explanation() {
        let result = Translator::validate_llm_output("I think you should run git status");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TranslationError::InvalidOutput(_)));

        let result2 = Translator::validate_llm_output("Here's what you need: git add .");
        assert!(result2.is_err());

        let result3 = Translator::validate_llm_output("To do this, run git commit");
        assert!(result3.is_err());
    }

    #[test]
    fn test_validate_llm_output_not_git_command() {
        let result = Translator::validate_llm_output("npm install");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TranslationError::InvalidOutput(_)));

        let result2 = Translator::validate_llm_output("ls -la");
        assert!(result2.is_err());

        let result3 = Translator::validate_llm_output("random text");
        assert!(result3.is_err());
    }

    #[test]
    fn test_validate_llm_output_shell_metacharacters() {
        // Should reject commands with shell metacharacters
        assert!(Translator::validate_llm_output("git status; rm -rf /").is_err());
        assert!(Translator::validate_llm_output("git log | cat").is_err());
        assert!(Translator::validate_llm_output("git status && ls").is_err());
        assert!(Translator::validate_llm_output("git status $(whoami)").is_err());
        assert!(Translator::validate_llm_output("git status `whoami`").is_err());
        assert!(Translator::validate_llm_output("git status > /tmp/out").is_err());
        assert!(Translator::validate_llm_output("git status < /tmp/in").is_err());
    }

    #[test]
    fn test_validate_llm_output_with_whitespace() {
        assert!(Translator::validate_llm_output("  git status  ").is_ok());
        assert!(Translator::validate_llm_output("\tgit add .\t").is_ok());
    }

    #[test]
    fn test_validate_llm_output_all_subcommands() {
        // Test all allowed subcommands
        let subcommands = [
            "status", "log", "show", "diff", "branch", "tag", "remote", "reflog",
            "blame", "describe", "add", "commit", "checkout", "switch", "restore",
            "reset", "revert", "merge", "rebase", "cherry-pick", "stash", "clean",
            "push", "pull", "fetch", "clone", "config", "filter-branch",
        ];

        for cmd in &subcommands {
            assert!(
                Translator::validate_llm_output(&format!("git {}", cmd)).is_ok(),
                "Should accept 'git {}'",
                cmd
            );
            assert!(
                Translator::validate_llm_output(cmd).is_ok(),
                "Should accept '{}'",
                cmd
            );
        }
    }

    #[tokio::test]
    async fn test_translator_rejects_invalid_llm_output() {
        use crate::git::Repository;

        if let Ok(repo) = Repository::discover() {
            let mock_client = Box::new(MockLLMClient {
                response: "I think you should run git status".to_string(),
            });

            let context_builder = ContextBuilder::new(repo);
            let translator = Translator::new(mock_client, context_builder);

            let result = translator.translate("show me the status").await;
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), TranslationError::InvalidOutput(_)));
        }
    }

    #[tokio::test]
    async fn test_translator_rejects_empty_output() {
        use crate::git::Repository;

        if let Ok(repo) = Repository::discover() {
            let mock_client = Box::new(MockLLMClient {
                response: "".to_string(),
            });

            let context_builder = ContextBuilder::new(repo);
            let translator = Translator::new(mock_client, context_builder);

            let result = translator.translate("show me the status").await;
            assert!(result.is_err());
        }
    }
}
