use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Failed to parse config file: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("Failed to serialize config: {0}")]
    SerializeError(#[from] toml::ser::Error),

    #[error("Config directory not found")]
    DirectoryNotFound,

    #[error("Invalid config value: {0}")]
    InvalidValue(String),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub llm: LLMConfig,
    pub ui: UIConfig,
    pub behavior: BehaviorConfig,
    pub git: GitConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LLMConfig {
    pub provider: String,
    pub model: String,
    pub api_key_env: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UIConfig {
    pub refresh_interval_ms: u64,
    pub max_commits_display: usize,
    pub max_stashes_display: usize,
    pub show_line_numbers: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BehaviorConfig {
    pub auto_refresh: bool,
    pub confirm_dangerous_ops: bool,
    pub log_commands: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GitConfig {
    pub timeout_seconds: u64,
}

impl Config {
    /// Get the config directory path
    pub fn config_dir() -> Result<PathBuf, ConfigError> {
        let home = std::env::var("HOME")
            .map_err(|_| ConfigError::DirectoryNotFound)?;
        Ok(PathBuf::from(home).join(".config").join("gitalky"))
    }

    /// Get the config file path
    pub fn config_path() -> Result<PathBuf, ConfigError> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    /// Load configuration from file
    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::config_path()?;

        if !path.exists() {
            return Err(ConfigError::ReadError(
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Config file not found"
                )
            ));
        }

        let contents = fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&contents)?;

        // Validate config
        config.validate()?;

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<(), ConfigError> {
        // Validate before saving
        self.validate()?;

        let dir = Self::config_dir()?;
        fs::create_dir_all(&dir)?;

        let path = Self::config_path()?;
        let contents = toml::to_string_pretty(self)?;

        fs::write(&path, contents)?;

        // Set permissions to 600 (owner read/write only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&path, perms)?;
        }

        Ok(())
    }

    /// Create default configuration
    pub fn default_config() -> Self {
        Config {
            llm: LLMConfig {
                provider: "anthropic".to_string(),
                model: "claude-sonnet-4-20250514".to_string(),
                api_key_env: "ANTHROPIC_API_KEY".to_string(),
                api_key: None,
            },
            ui: UIConfig {
                refresh_interval_ms: 100,
                max_commits_display: 5,
                max_stashes_display: 5,
                show_line_numbers: false,
            },
            behavior: BehaviorConfig {
                auto_refresh: true,
                confirm_dangerous_ops: true,
                log_commands: true,
            },
            git: GitConfig {
                timeout_seconds: 30,
            },
        }
    }

    /// Validate configuration values
    fn validate(&self) -> Result<(), ConfigError> {
        // Validate provider
        if self.llm.provider != "anthropic" {
            return Err(ConfigError::InvalidValue(
                format!("Unsupported LLM provider: {}. Only 'anthropic' is supported in v1",
                    self.llm.provider)
            ));
        }

        // Validate model
        if !self.llm.model.starts_with("claude-") {
            return Err(ConfigError::InvalidValue(
                format!("Invalid model name: {}. Must be a Claude model", self.llm.model)
            ));
        }

        // Validate UI settings
        if self.ui.refresh_interval_ms == 0 {
            return Err(ConfigError::InvalidValue(
                "refresh_interval_ms must be greater than 0".to_string()
            ));
        }

        if self.ui.max_commits_display == 0 {
            return Err(ConfigError::InvalidValue(
                "max_commits_display must be greater than 0".to_string()
            ));
        }

        // Validate git timeout
        if self.git.timeout_seconds == 0 {
            return Err(ConfigError::InvalidValue(
                "timeout_seconds must be greater than 0".to_string()
            ));
        }

        Ok(())
    }

    /// Get API key from environment variable or config
    pub fn get_api_key(&self) -> Option<String> {
        // First try environment variable
        if let Ok(key) = std::env::var(&self.llm.api_key_env) {
            if !key.is_empty() {
                return Some(key);
            }
        }

        // Fall back to config file if present
        self.llm.api_key.clone()
    }

    /// Check if API key is available
    pub fn has_api_key(&self) -> bool {
        self.get_api_key().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default_config();
        assert_eq!(config.llm.provider, "anthropic");
        assert!(config.llm.model.starts_with("claude-"));
        assert_eq!(config.llm.api_key_env, "ANTHROPIC_API_KEY");
        assert!(config.behavior.confirm_dangerous_ops);
    }

    #[test]
    fn test_validate_valid_config() {
        let config = Config::default_config();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_provider() {
        let mut config = Config::default_config();
        config.llm.provider = "openai".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_model() {
        let mut config = Config::default_config();
        config.llm.model = "gpt-4".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_zero_refresh_interval() {
        let mut config = Config::default_config();
        config.ui.refresh_interval_ms = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_api_key_from_env() {
        unsafe {
            std::env::set_var("TEST_API_KEY", "test-key-123");
        }
        let mut config = Config::default_config();
        config.llm.api_key_env = "TEST_API_KEY".to_string();

        assert_eq!(config.get_api_key(), Some("test-key-123".to_string()));
        assert!(config.has_api_key());

        unsafe {
            std::env::remove_var("TEST_API_KEY");
        }
    }

    #[test]
    fn test_api_key_from_config() {
        let mut config = Config::default_config();
        config.llm.api_key_env = "NONEXISTENT_VAR".to_string();
        config.llm.api_key = Some("config-key-456".to_string());

        assert_eq!(config.get_api_key(), Some("config-key-456".to_string()));
        assert!(config.has_api_key());
    }

    #[test]
    fn test_serialize_deserialize() {
        let config = Config::default_config();
        let toml = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&toml).unwrap();

        assert_eq!(config.llm.provider, parsed.llm.provider);
        assert_eq!(config.llm.model, parsed.llm.model);
    }
}
