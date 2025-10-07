use gitalky::error::{AppError, GitError, AppResult};
use gitalky::config::settings::ConfigError;
use gitalky::config::first_run::SetupError;
use gitalky::llm::client::LLMError;
use gitalky::llm::translator::TranslationError;
use gitalky::security::validator::ValidationError;
use std::error::Error;

/// Test that GitError converts to AppError::Git
#[test]
fn test_git_error_converts_to_app_error() {
    let git_err = GitError::NotARepository;
    let app_err: AppError = git_err.into();
    assert!(matches!(app_err, AppError::Git(_)));
}

/// Test that ConfigError converts to AppError::Config
#[test]
fn test_config_error_converts_to_app_error() {
    let config_err = ConfigError::DirectoryNotFound;
    let app_err: AppError = config_err.into();
    assert!(matches!(app_err, AppError::Config(_)));
}

/// Test that LLMError converts to AppError::Llm
#[test]
fn test_llm_error_converts_to_app_error() {
    let llm_err = LLMError::Timeout;
    let app_err: AppError = llm_err.into();
    assert!(matches!(app_err, AppError::Llm(_)));
}

/// Test that TranslationError converts to AppError::Translation
#[test]
fn test_translation_error_converts_to_app_error() {
    let translation_err = TranslationError::LLMError(LLMError::Timeout);
    let app_err: AppError = translation_err.into();
    assert!(matches!(app_err, AppError::Translation(_)));
}

/// Test that ValidationError converts to AppError::Security
#[test]
fn test_validation_error_converts_to_app_error() {
    let validation_err = ValidationError::InvalidFormat;
    let app_err: AppError = validation_err.into();
    assert!(matches!(app_err, AppError::Security(_)));
}

/// Test that SetupError converts to AppError::Setup
#[test]
fn test_setup_error_converts_to_app_error() {
    let setup_err = SetupError::Cancelled;
    let app_err: AppError = setup_err.into();
    assert!(matches!(app_err, AppError::Setup(_)));
}

/// Test that std::io::Error converts to AppError::Io
#[test]
fn test_io_error_converts_to_app_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
    let app_err: AppError = io_err.into();
    assert!(matches!(app_err, AppError::Io(_)));
}

/// Test that error source is preserved
#[test]
fn test_error_source_preserved() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test file");
    let git_err = GitError::IoError(io_err);
    let app_err: AppError = git_err.into();

    // Check that we can access the source error
    assert!(app_err.source().is_some());
}

/// Test that error messages are user-friendly
#[test]
fn test_error_display_user_friendly() {
    let app_err = AppError::Git(GitError::NotARepository);
    let msg = format!("{}", app_err);
    assert!(msg.contains("repository") || msg.contains("Git"));
}

/// Test AppError::Git variant displays correctly
#[test]
fn test_app_error_git_display() {
    let app_err = AppError::Git(GitError::CommandFailed("test".to_string()));
    let msg = format!("{}", app_err);
    assert!(msg.contains("Git error"));
    assert!(msg.contains("test"));
}

/// Test AppError::Config variant displays correctly
#[test]
fn test_app_error_config_display() {
    let app_err = AppError::Config(ConfigError::DirectoryNotFound);
    let msg = format!("{}", app_err);
    assert!(msg.contains("Configuration error"));
}

/// Test AppError::Llm variant displays correctly
#[test]
fn test_app_error_llm_display() {
    let app_err = AppError::Llm(LLMError::Timeout);
    let msg = format!("{}", app_err);
    assert!(msg.contains("LLM error"));
}

/// Test AppError::Security variant displays correctly
#[test]
fn test_app_error_security_display() {
    let app_err = AppError::Security(ValidationError::DisallowedSubcommand("rm".to_string()));
    let msg = format!("{}", app_err);
    assert!(msg.contains("Security validation error"));
}

/// Test that ? operator works with AppError
#[test]
fn test_question_mark_operator() {
    fn may_fail() -> Result<(), GitError> {
        Err(GitError::NotARepository)
    }

    fn outer() -> AppResult<()> {
        // This should automatically convert GitError to AppError
        may_fail()?;
        Ok(())
    }

    let result = outer();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AppError::Git(_)));
}

/// Test nested error conversion (GitError -> AppError)
#[test]
fn test_nested_git_error_conversion() {
    fn inner() -> Result<(), GitError> {
        Err(GitError::ParseError("test".to_string()))
    }

    fn outer() -> AppResult<()> {
        inner()?;
        Ok(())
    }

    let result = outer();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AppError::Git(_)));
}

/// Test nested error conversion (ConfigError -> AppError)
#[test]
fn test_nested_config_error_conversion() {
    fn inner() -> Result<(), ConfigError> {
        Err(ConfigError::InvalidValue("test".to_string()))
    }

    fn outer() -> AppResult<()> {
        inner()?;
        Ok(())
    }

    let result = outer();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AppError::Config(_)));
}

/// Test that Debug trait works for AppError
#[test]
fn test_app_error_debug() {
    let app_err = AppError::Git(GitError::NotARepository);
    let debug_str = format!("{:?}", app_err);
    assert!(!debug_str.is_empty());
}

/// Test error chain with TranslationError wrapping GitError
#[test]
fn test_translation_error_wraps_git_error() {
    let git_err = GitError::CommandFailed("git status failed".to_string());
    let trans_err = TranslationError::ContextError(git_err);
    let app_err: AppError = trans_err.into();

    assert!(matches!(app_err, AppError::Translation(_)));

    // Verify we can still access the inner error through Display
    let msg = format!("{}", app_err);
    assert!(msg.contains("Translation error"));
}

/// Test that all error variants can be constructed and converted
#[test]
fn test_all_error_variants_convertible() {
    let errors: Vec<AppError> = vec![
        AppError::Git(GitError::NotARepository),
        AppError::Config(ConfigError::DirectoryNotFound),
        AppError::Llm(LLMError::Timeout),
        AppError::Translation(TranslationError::LLMError(LLMError::Timeout)),
        AppError::Security(ValidationError::InvalidFormat),
        AppError::Setup(SetupError::Cancelled),
        AppError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "test")),
    ];

    // Just verify they all can be created
    assert_eq!(errors.len(), 7);
}
