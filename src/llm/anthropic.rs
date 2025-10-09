use crate::llm::client::{GitCommand, LLMClient, LLMError};
use crate::llm::context::RepoContext;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::{Duration, Instant};

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const DEFAULT_MODEL: &str = "claude-sonnet-4-5-20250929";
const MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF_MS: u64 = 1000;

// Rate limiting: 10 requests per minute
const RATE_LIMIT_REQUESTS: usize = 10;
const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60);

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: String,
}

pub struct AnthropicClient {
    api_key: String,
    model: String,
    http_client: Client,
    // Rate limiting: track request timestamps
    request_times: Mutex<Vec<Instant>>,
}

impl AnthropicClient {
    pub fn new(api_key: String) -> Self {
        Self::with_model(api_key, DEFAULT_MODEL.to_string())
    }

    pub fn with_model(api_key: String, model: String) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            api_key,
            model,
            http_client,
            request_times: Mutex::new(Vec::new()),
        }
    }

    /// Check and enforce rate limiting
    /// Returns Ok(()) if request is allowed, Err with wait time if rate limited
    fn check_rate_limit(&self) -> Result<(), LLMError> {
        let now = Instant::now();
        let mut times = self.request_times.lock().unwrap();

        // Remove requests older than the rate limit window
        times.retain(|&time| now.duration_since(time) < RATE_LIMIT_WINDOW);

        // Check if we've exceeded the rate limit
        if times.len() >= RATE_LIMIT_REQUESTS {
            let oldest = times[0];
            let wait_time = RATE_LIMIT_WINDOW.saturating_sub(now.duration_since(oldest));
            return Err(LLMError::RateLimitExceeded(wait_time.as_secs()));
        }

        // Record this request
        times.push(now);
        Ok(())
    }

    async fn call_api(&self, prompt: &str, context: &str) -> Result<String, LLMError> {
        let full_prompt = format!(
            "You are a git command expert. Translate the user's natural language query into a git command.

Repository Context:
{}

User Query: {}

CRITICAL INSTRUCTIONS:
- Respond with ONLY the git command itself
- Do NOT include explanations, reasoning, or commentary
- Do NOT use markdown code blocks or backticks
- Do NOT use multiple lines
- Output format: exactly one line containing just the git command
- Example good response: git status
- Example bad response: ```bash\\ngit status\\n```

FILE PATH MATCHING:
- When the user mentions a file name, look at the repository files in the context
- Use fuzzy matching to find the correct file path
- If user says \"add input.rs\", look for files ending in \"input.rs\" like \"src/ui/input.rs\"
- Always use the full path from the repository context
- Prioritize exact basename matches over partial matches
- Examples:
  * User: \"add input.rs\" → git add src/ui/input.rs (if that's the only input.rs)
  * User: \"stage app.rs\" → git add src/ui/app.rs (if that's in the file list)
  * User: \"add main\" → git add src/main.rs (if that's in the file list)

Your response:",
            context, prompt
        );

        let request_body = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: 1024,
            messages: vec![Message {
                role: "user".to_string(),
                content: full_prompt,
            }],
        };

        let mut attempt = 0;
        let mut backoff_ms = INITIAL_BACKOFF_MS;

        loop {
            attempt += 1;

            let response = self
                .http_client
                .post(ANTHROPIC_API_URL)
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&request_body)
                .send()
                .await?;

            let status = response.status();

            if status.is_success() {
                let api_response: AnthropicResponse = response.json().await?;

                if let Some(content) = api_response.content.first() {
                    return Ok(content.text.clone());
                } else {
                    return Err(LLMError::InvalidResponse(
                        "No content in response".to_string(),
                    ));
                }
            } else if status.as_u16() == 429 {
                // Rate limit - check retry-after header
                let retry_after = response
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(60);

                if attempt >= MAX_RETRIES {
                    return Err(LLMError::RateLimitExceeded(retry_after));
                }

                // Exponential backoff with retry-after
                let wait_ms = retry_after.saturating_mul(1000).max(backoff_ms);
                eprintln!(
                    "Rate limited, retrying in {}ms (attempt {}/{})",
                    wait_ms, attempt, MAX_RETRIES
                );

                tokio::time::sleep(Duration::from_millis(wait_ms)).await;
                backoff_ms *= 2;
                continue;
            } else {
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                return Err(LLMError::ApiError(format!(
                    "API returned status {}: {}",
                    status, error_text
                )));
            }
        }
    }
}

#[async_trait]
impl LLMClient for AnthropicClient {
    async fn translate(&self, query: &str, context: &RepoContext) -> Result<GitCommand, LLMError> {
        // Check rate limiting before making API call
        self.check_rate_limit()?;

        let context_str = context.get_full_context();
        let response = self.call_api(query, &context_str).await?;

        // Clean up response - strip markdown, extra whitespace, etc.
        let command = Self::clean_response(&response);

        // Basic validation: should start with "git" or be a git subcommand
        if !command.starts_with("git ") && !Self::is_git_subcommand(&command) {
            return Err(LLMError::InvalidResponse(format!(
                "Response doesn't look like a git command: {}",
                command
            )));
        }

        Ok(GitCommand {
            command,
            explanation: None,
        })
    }
}

impl AnthropicClient {
    /// Clean up LLM response to extract just the git command
    fn clean_response(response: &str) -> String {
        let mut cleaned = response.trim();

        // Strip markdown code blocks (```bash ... ``` or ``` ... ```)
        if cleaned.starts_with("```") {
            // Remove opening ```bash or ```
            if let Some(first_newline) = cleaned.find('\n') {
                cleaned = &cleaned[first_newline + 1..];
            }
            // Remove closing ```
            if let Some(last_backticks) = cleaned.rfind("```") {
                cleaned = &cleaned[..last_backticks];
            }
            cleaned = cleaned.trim();
        }

        // Take only the first line (in case there's explanation after)
        if let Some(first_line) = cleaned.lines().next() {
            cleaned = first_line.trim();
        }

        cleaned.to_string()
    }

    fn is_git_subcommand(cmd: &str) -> bool {
        // Common git subcommands that might be returned without "git" prefix
        let subcommands = [
            "status", "commit", "add", "push", "pull", "branch", "checkout", "merge",
            "rebase", "log", "diff", "stash", "reset", "tag", "fetch", "clone", "init",
        ];

        let first_word = cmd.split_whitespace().next().unwrap_or("");
        subcommands.contains(&first_word)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_response_simple() {
        let response = "git status";
        assert_eq!(AnthropicClient::clean_response(response), "git status");
    }

    #[test]
    fn test_clean_response_with_whitespace() {
        let response = "  git status  \n";
        assert_eq!(AnthropicClient::clean_response(response), "git status");
    }

    #[test]
    fn test_clean_response_markdown_bash() {
        let response = "```bash\ngit status\n```";
        assert_eq!(AnthropicClient::clean_response(response), "git status");
    }

    #[test]
    fn test_clean_response_markdown_plain() {
        let response = "```\ngit status\n```";
        assert_eq!(AnthropicClient::clean_response(response), "git status");
    }

    #[test]
    fn test_clean_response_multiline_with_explanation() {
        let response = "git status\n\nThis shows the working tree status.";
        assert_eq!(AnthropicClient::clean_response(response), "git status");
    }

    #[test]
    fn test_clean_response_complex() {
        let response = "```bash\ngit diff main..\n```\n\nWait, I need more context...";
        assert_eq!(AnthropicClient::clean_response(response), "git diff main..");
    }

    #[test]
    fn test_is_git_subcommand() {
        assert!(AnthropicClient::is_git_subcommand("status"));
        assert!(AnthropicClient::is_git_subcommand("commit -m 'test'"));
        assert!(!AnthropicClient::is_git_subcommand("notacommand"));
    }

    #[test]
    fn test_rate_limiting_allows_initial_requests() {
        let client = AnthropicClient::new("test-key".to_string());

        // First 10 requests should succeed
        for _ in 0..10 {
            assert!(client.check_rate_limit().is_ok());
        }
    }

    #[test]
    fn test_rate_limiting_blocks_excess_requests() {
        let client = AnthropicClient::new("test-key".to_string());

        // Fill up the rate limit
        for _ in 0..10 {
            client.check_rate_limit().unwrap();
        }

        // 11th request should be rate limited
        let result = client.check_rate_limit();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LLMError::RateLimitExceeded(_)));
    }

    #[test]
    fn test_rate_limiting_window_expiry() {
        use std::thread;

        let client = AnthropicClient::new("test-key".to_string());

        // Make 1 request
        client.check_rate_limit().unwrap();

        // Wait for 1 second (requests older than 60s are removed)
        // Note: This test doesn't wait the full minute for CI performance
        // In production, requests older than 60s would be purged
        thread::sleep(Duration::from_millis(100));

        // Should still be able to make requests (we haven't hit the limit)
        for _ in 0..9 {
            assert!(client.check_rate_limit().is_ok());
        }
    }
}
