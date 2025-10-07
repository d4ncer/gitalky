# Error Handling in Gitalky

## Overview

Gitalky uses a unified error handling architecture with a top-level `AppError` type that wraps all module-specific errors. This design provides:

- **Type Safety**: Compile-time guarantees about error handling
- **Automatic Conversion**: `From` trait implementations for seamless error propagation
- **Error Context Preservation**: Full error chains accessible via `source()`
- **User-Friendly Messages**: Translation layer for end-user error display

## Error Types

### AppError (Top-Level)

The unified application-level error type that wraps all module-specific errors:

```rust
pub enum AppError {
    Git(GitError),              // Git operations
    Config(ConfigError),         // Configuration errors
    Llm(LLMError),              // LLM communication errors
    Translation(TranslationError), // Query translation errors
    Security(ValidationError),   // Command validation errors
    Setup(SetupError),          // First-run setup errors
    Io(io::Error),              // I/O operations
}
```

### GitError (Module-Level)

Git-specific errors for repository operations:

```rust
pub enum GitError {
    NotARepository,
    CommandFailed(String),
    ParseError(String),
    GitVersionTooOld(String),
    GitVersionDetectionFailed(String),
    IoError(io::Error),
}
```

## Type Aliases

Use these type aliases for function return types:

```rust
pub type AppResult<T> = std::result::Result<T, AppError>;
pub type GitResult<T> = std::result::Result<T, GitError>;
```

## Usage Patterns

### 1. Module-Level Functions (Git Module)

Functions in the git module should return `GitResult<T>`:

```rust
pub fn discover() -> GitResult<Self> {
    // Implementation
}

pub fn execute(&self, command: &str) -> GitResult<CommandOutput> {
    // Implementation
}
```

### 2. Application-Level Functions

Functions at the application level (UI, main) should return `AppResult<T>`:

```rust
pub fn new(repo: Repository, config: Config) -> AppResult<Self> {
    let api_key = config.llm.api_key.clone()
        .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
        .ok_or_else(|| {
            ConfigError::InvalidValue(
                "No API key found in config or environment".to_string()
            )
        })?;

    // Implementation
}
```

### 3. Error Propagation with `?` Operator

The `?` operator automatically converts module-specific errors to `AppError`:

```rust
pub async fn try_reconnect(&mut self) -> AppResult<()> {
    // GitError automatically converts to AppError::Git
    self.repo.refresh()?;

    // ConfigError automatically converts to AppError::Config
    let config = Config::load()?;

    Ok(())
}
```

### 4. Manual Error Creation

When you need to create errors explicitly:

```rust
// Create a ConfigError that will convert to AppError
return Err(ConfigError::InvalidValue(
    "No API key found".to_string()
).into());

// Create a GitError that will convert to AppError
return Err(GitError::CommandFailed("git command failed".to_string()).into());
```

### 5. Nested Error Handling

Preserve error chains for debugging:

```rust
pub fn build_context(&self) -> AppResult<RepoContext> {
    // Inner function returns GitResult
    fn get_status() -> GitResult<String> {
        // May fail with GitError
    }

    // Outer function returns AppResult
    // GitError automatically converts to AppError::Git
    let status = get_status()?;

    Ok(RepoContext { status })
}
```

## Error Translation

### User-Friendly Error Messages

The `ErrorTranslator` provides user-friendly error messages:

```rust
use crate::error_translation::translator::ErrorTranslator;

let app_error: AppError = some_operation()?;
let friendly = ErrorTranslator::translate_app_error(&app_error);

println!("{}", friendly.simple_message);
if let Some(suggestion) = friendly.suggestion {
    println!("Suggestion: {}", suggestion);
}
```

### Example Translations

**Git Error:**
```
Raw: "fatal: The current branch has no upstream branch"
Translated: "No remote branch is configured for tracking."
Suggestion: "Try: git push -u origin <branch-name>"
```

**Config Error:**
```
Raw: "DirectoryNotFound"
Translated: "Configuration error occurred."
Suggestion: "Check your config file at ~/.config/gitalky/config.toml"
```

**LLM Error:**
```
Raw: "Timeout"
Translated: "Error communicating with LLM."
Suggestion: "Check your API key and network connection"
```

## Best Practices

### DO ✅

1. **Use type aliases consistently:**
   ```rust
   pub fn my_function() -> GitResult<String> { /* ... */ }
   pub fn app_function() -> AppResult<()> { /* ... */ }
   ```

2. **Let `From` trait do the conversion:**
   ```rust
   pub fn outer() -> AppResult<()> {
       inner_git_function()?;  // Auto-converts GitError to AppError
       Ok(())
   }
   ```

3. **Preserve error context:**
   ```rust
   let io_err = std::io::Error::new(ErrorKind::NotFound, "file.txt");
   let git_err = GitError::IoError(io_err);  // Wraps io::Error
   let app_err: AppError = git_err.into();   // Preserves chain
   assert!(app_err.source().is_some());      // Can access original
   ```

4. **Use appropriate error variants:**
   ```rust
   // For config issues
   Err(ConfigError::InvalidValue("Bad value".to_string()).into())

   // For security validation
   Err(ValidationError::DisallowedSubcommand("rm".to_string()).into())
   ```

### DON'T ❌

1. **Don't use generic error messages:**
   ```rust
   // Bad
   return Err(GitError::CommandFailed("error".to_string()));

   // Good
   return Err(GitError::CommandFailed(format!(
       "Failed to checkout branch '{}': {}",
       branch_name, stderr
   )));
   ```

2. **Don't swallow error context:**
   ```rust
   // Bad
   match some_operation() {
       Ok(val) => val,
       Err(_) => return Err(GitError::CommandFailed("failed".to_string())),
   }

   // Good
   some_operation()?  // Preserves full error context
   ```

3. **Don't mix Result types incorrectly:**
   ```rust
   // Bad - git module returning AppResult
   pub fn git_status() -> AppResult<String> { /* ... */ }

   // Good - git module returns GitResult
   pub fn git_status() -> GitResult<String> { /* ... */ }
   ```

## Testing

### Testing Error Conversions

```rust
#[test]
fn test_git_error_converts_to_app_error() {
    let git_err = GitError::NotARepository;
    let app_err: AppError = git_err.into();
    assert!(matches!(app_err, AppError::Git(_)));
}

#[test]
fn test_question_mark_operator() {
    fn may_fail() -> GitResult<()> {
        Err(GitError::NotARepository)
    }

    fn outer() -> AppResult<()> {
        may_fail()?;  // Auto-converts GitError to AppError
        Ok(())
    }

    let result = outer();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AppError::Git(_)));
}
```

### Testing Error Messages

```rust
#[test]
fn test_error_display_user_friendly() {
    let app_err = AppError::Git(GitError::NotARepository);
    let msg = format!("{}", app_err);
    assert!(msg.contains("repository") || msg.contains("Git"));
}

#[test]
fn test_error_translation() {
    let error = GitError::CommandFailed(
        "fatal: The current branch has no upstream branch".to_string()
    );
    let translated = ErrorTranslator::translate(&error);

    assert!(translated.simple_message.contains("No remote branch"));
    assert!(translated.suggestion.is_some());
}
```

## Migration Guide

### Converting Old Code to New Error Architecture

**Before:**
```rust
pub fn my_function() -> Result<String, GitError> {
    // ...
}

// Error creation
return Err(GitError::Custom("Something failed".to_string()));
```

**After:**
```rust
pub fn my_function() -> GitResult<String> {
    // ...
}

// Error creation - use appropriate variant
return Err(GitError::CommandFailed("Something failed".to_string()));

// Or use a more specific error type
return Err(ConfigError::InvalidValue("Bad config".to_string()).into());
```

## Error Hierarchy

```
AppError (Application Level)
├── Git(GitError)
│   ├── NotARepository
│   ├── CommandFailed(String)
│   ├── ParseError(String)
│   ├── GitVersionTooOld(String)
│   ├── GitVersionDetectionFailed(String)
│   └── IoError(io::Error)
├── Config(ConfigError)
│   ├── DirectoryNotFound
│   ├── InvalidValue(String)
│   └── ...
├── Llm(LLMError)
│   ├── Timeout
│   └── ...
├── Translation(TranslationError)
├── Security(ValidationError)
│   ├── InvalidFormat
│   ├── DisallowedSubcommand(String)
│   └── ...
├── Setup(SetupError)
│   └── Cancelled
└── Io(io::Error)
```

## References

- Source: `src/error.rs`
- Tests: `tests/error_conversions.rs`
- Translation: `src/error_translation/translator.rs`
- Spec: `codev/specs/0003-tech-debt-cleanup.md`
- Plan: `codev/plans/0003-tech-debt-cleanup.md`
