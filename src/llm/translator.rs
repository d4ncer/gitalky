use crate::llm::client::{GitCommand, LLMClient, LLMError};
use crate::llm::context::ContextBuilder;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TranslationError {
    #[error("LLM error: {0}")]
    LLMError(#[from] LLMError),

    #[error("Context building error: {0}")]
    ContextError(#[from] crate::error::GitError),
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

        Ok(command)
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
}
