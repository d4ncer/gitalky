use super::settings::{Config, ConfigError};
use std::io::{self, Write};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SetupError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Config error: {0}")]
    ConfigError(#[from] ConfigError),

    #[error("Setup cancelled by user")]
    Cancelled,

    #[error("API validation failed: {0}")]
    ValidationFailed(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SetupStep {
    Welcome,
    SelectProvider,
    SelectKeySource,
    EnterAPIKey,
    ValidateAPI,
    Complete,
}

pub struct FirstRunWizard {
    step: SetupStep,
    config: Config,
}

impl FirstRunWizard {
    pub fn new() -> Self {
        FirstRunWizard {
            step: SetupStep::Welcome,
            config: Config::default_config(),
        }
    }

    /// Run the first-run wizard and return configured Config
    pub async fn run() -> Result<Config, SetupError> {
        let mut wizard = Self::new();

        // Welcome
        wizard.show_welcome()?;
        wizard.step = SetupStep::SelectProvider;

        // Select provider
        let use_llm = wizard.select_provider()?;
        if !use_llm {
            // Skip LLM setup - offline mode
            wizard.config.llm.api_key_env = String::new();
            wizard.step = SetupStep::Complete;
            wizard.show_complete(false)?;
            return Ok(wizard.config);
        }

        wizard.step = SetupStep::SelectKeySource;

        // Select key source (env var or direct input)
        let use_env = wizard.select_key_source()?;

        if use_env {
            // Use environment variable
            wizard.step = SetupStep::ValidateAPI;
            wizard.validate_api().await?;
        } else {
            // Enter API key directly
            wizard.step = SetupStep::EnterAPIKey;
            let api_key = wizard.enter_api_key()?;
            wizard.config.llm.api_key = Some(api_key);

            wizard.step = SetupStep::ValidateAPI;
            wizard.validate_api().await?;
        }

        wizard.step = SetupStep::Complete;
        wizard.show_complete(true)?;

        Ok(wizard.config)
    }

    fn show_welcome(&self) -> Result<(), SetupError> {
        println!("\n{}", "=".repeat(70));
        println!(r#"
   _____ _ _        _ _
  / ____(_) |      | | |
 | |  __ _| |_ __ _| | | ___   _
 | | |_ | | __/ _` | | |/ / | | |
 | |__| | | || (_| | |   <| |_| |
  \_____|_|\__\__,_|_|_|\_\\__, |
                            __/ |
                           |___/
        "#);
        println!("  Natural Language Git Terminal UI");
        println!("{}", "=".repeat(70));
        println!("\nWelcome to Gitalky!");
        println!("\nThis wizard will help you configure Gitalky for first use.");
        println!("Press Enter to continue...");

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        Ok(())
    }

    fn select_provider(&mut self) -> Result<bool, SetupError> {
        println!("\n{}", "-".repeat(70));
        println!("LLM Provider Selection");
        println!("{}", "-".repeat(70));
        println!("\nGitalky uses AI to translate natural language into git commands.");
        println!("\nSelect your LLM provider:");
        println!("  [1] Anthropic Claude (recommended)");
        println!("  [2] OpenAI (coming soon)");
        println!("  [3] Local/Ollama (coming soon)");
        println!("  [4] Skip - Use offline mode (direct git commands only)");
        print!("\nEnter your choice [1-4]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim();

        match choice {
            "1" => {
                self.config.llm.provider = "anthropic".to_string();
                Ok(true)
            }
            "2" | "3" => {
                println!("\n⚠️  This provider is not yet supported in v1.");
                println!("Please select option [1] for Anthropic or [4] to skip.");
                self.select_provider()
            }
            "4" => {
                println!("\n✓ Offline mode selected. You can configure an LLM later.");
                Ok(false)
            }
            _ => {
                println!("\n⚠️  Invalid choice. Please enter 1-4.");
                self.select_provider()
            }
        }
    }

    fn select_key_source(&mut self) -> Result<bool, SetupError> {
        println!("\n{}", "-".repeat(70));
        println!("API Key Configuration");
        println!("{}", "-".repeat(70));
        println!("\nHow would you like to provide your Anthropic API key?");
        println!("\n  [1] Environment variable (recommended - more secure)");
        println!("  [2] Store in config file (less secure but convenient)");
        print!("\nEnter your choice [1-2]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim();

        match choice {
            "1" => {
                println!("\n✓ Using environment variable.");
                println!("\nPlease ensure the ANTHROPIC_API_KEY environment variable is set:");
                println!("  export ANTHROPIC_API_KEY='your-api-key-here'");
                println!("\nPress Enter when ready to continue...");
                let mut _input = String::new();
                io::stdin().read_line(&mut _input)?;
                Ok(true)
            }
            "2" => {
                println!("\n⚠️  Warning: Storing API keys in config files is less secure.");
                println!("The config file will be set to permissions 600 (owner read/write only).");
                Ok(false)
            }
            _ => {
                println!("\n⚠️  Invalid choice. Please enter 1 or 2.");
                self.select_key_source()
            }
        }
    }

    fn enter_api_key(&self) -> Result<String, SetupError> {
        println!("\n{}", "-".repeat(70));
        println!("Enter API Key");
        println!("{}", "-".repeat(70));
        print!("\nEnter your Anthropic API key: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let api_key = input.trim().to_string();

        if api_key.is_empty() {
            println!("\n⚠️  API key cannot be empty.");
            return self.enter_api_key();
        }

        Ok(api_key)
    }

    async fn validate_api(&self) -> Result<(), SetupError> {
        println!("\n{}", "-".repeat(70));
        println!("Validating API Connection");
        println!("{}", "-".repeat(70));
        println!("\nTesting API connection...");

        // Try to get API key
        let api_key = self.config.get_api_key();
        if api_key.is_none() {
            println!("\n⚠️  No API key found.");
            println!("\nWould you like to:");
            println!("  [1] Try again");
            println!("  [2] Skip validation and continue");
            print!("\nEnter your choice [1-2]: ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let choice = input.trim();

            match choice {
                "1" => return Err(SetupError::ValidationFailed("No API key found".to_string())),
                "2" => {
                    println!("\n⚠️  Skipping validation. You may need to configure this later.");
                    return Ok(());
                }
                _ => {
                    return Box::pin(self.validate_api()).await;
                }
            }
        }

        // Perform actual validation using a simple API test
        let api_key = api_key.unwrap();
        match Self::test_api_connection(&api_key, &self.config.llm.model).await {
            Ok(_) => {
                println!("\n✓ API connection successful!");
                Ok(())
            }
            Err(e) => {
                println!("\n⚠️  API validation failed: {}", e);
                println!("\nWould you like to:");
                println!("  [1] Try again with a different key");
                println!("  [2] Skip validation and continue");
                print!("\nEnter your choice [1-2]: ");
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let choice = input.trim();

                match choice {
                    "1" => Err(SetupError::ValidationFailed(format!("Validation failed: {}", e))),
                    "2" => {
                        println!("\n⚠️  Skipping validation. The app may not work correctly.");
                        Ok(())
                    }
                    _ => Box::pin(self.validate_api()).await,
                }
            }
        }
    }

    async fn test_api_connection(api_key: &str, model: &str) -> Result<(), String> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let request_body = serde_json::json!({
            "model": model,
            "max_tokens": 10,
            "messages": [{
                "role": "user",
                "content": "test"
            }]
        });

        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("API returned status {}: {}", status, body))
        }
    }

    fn show_complete(&self, with_llm: bool) -> Result<(), SetupError> {
        println!("\n{}", "=".repeat(70));
        println!("Setup Complete!");
        println!("{}", "=".repeat(70));

        if with_llm {
            println!("\n✓ Gitalky is configured and ready to use!");
            println!("\nYou can now use natural language to interact with git:");
            println!("  - \"show me what changed\"");
            println!("  - \"commit these changes with message 'fix bug'\"");
            println!("  - \"create a new branch called feature-x\"");
        } else {
            println!("\n✓ Gitalky is configured in offline mode.");
            println!("\nYou can use direct git commands in the TUI.");
            println!("To enable AI features later, edit:");
            println!("  ~/.config/gitalky/config.toml");
        }

        println!("\nConfiguration saved to: ~/.config/gitalky/config.toml");
        println!("Audit log will be saved to: ~/.config/gitalky/history.log");
        println!("\nPress '?' in the app for help.");
        println!("\nPress Enter to start Gitalky...");

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wizard_creation() {
        let wizard = FirstRunWizard::new();
        assert_eq!(wizard.step, SetupStep::Welcome);
    }

    #[test]
    fn test_setup_steps() {
        assert_ne!(SetupStep::Welcome, SetupStep::Complete);
        assert_eq!(SetupStep::Welcome, SetupStep::Welcome);
    }
}
