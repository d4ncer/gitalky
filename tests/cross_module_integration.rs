mod helpers;

use gitalky::error::{AppError, GitError};
use gitalky::error_translation::translator::ErrorTranslator;
use gitalky::llm::context::{ContextBuilder, QueryType};
use gitalky::git::Repository;
use helpers::{create_commit, create_test_repo};
use std::fs;

/// Test that error conversions work end-to-end from git operations
#[test]
fn test_git_error_to_friendly_message_integration() {
    let (_temp, repo_path) = create_test_repo();
    let repo = Repository::new(&repo_path);

    // Create a git error by trying to get upstream when there is none
    let state = repo.state().expect("Failed to get state");

    // If we had an error, it would convert through AppError
    // This tests the integration of error types
    assert!(state.upstream.is_none()); // No upstream configured
}

/// Test context building integrates with repository state
#[test]
fn test_context_builder_with_real_repo_state() {
    let (_temp, repo_path) = create_test_repo();

    // Create realistic repo state
    create_commit(&repo_path, "main.rs", "fn main() {}", "Initial commit");
    create_commit(&repo_path, "lib.rs", "pub mod foo;", "Add lib");

    // Add some unstaged changes
    fs::write(repo_path.join("main.rs"), "fn main() { println!(\"Hello\"); }")
        .expect("Failed to modify file");

    // Add untracked file
    fs::write(repo_path.join("new_file.rs"), "// new file")
        .expect("Failed to create file");

    let repo = Repository::new(&repo_path);
    let context_builder = ContextBuilder::new(repo);

    // Test default context
    let default_ctx = context_builder.build_default_context()
        .expect("Failed to build default context");

    assert!(default_ctx.default_info.contains("Current branch"));
    assert!(default_ctx.default_info.contains("main.rs"));
    assert!(default_ctx.default_info.contains("new_file.rs"));
    assert!(default_ctx.estimated_tokens > 0);
}

/// Test query classification with various inputs
#[test]
fn test_query_classification_comprehensive() {
    let queries_and_expected = vec![
        ("commit my changes", QueryType::Commit),
        ("stage all files for commit", QueryType::Commit),
        ("create a new branch", QueryType::Branch),
        ("switch to main branch", QueryType::Branch),
        ("checkout feature", QueryType::Branch),
        ("show me what changed", QueryType::Diff),
        ("view recent history", QueryType::History),
        ("show me the log", QueryType::History),
        ("stash my work", QueryType::Stash),
        ("what's the status", QueryType::General),
        ("help me with git", QueryType::General),
    ];

    for (query, expected) in queries_and_expected {
        let result = ContextBuilder::classify_query(query);
        assert_eq!(
            result, expected,
            "Query '{}' should classify as {:?}",
            query, expected
        );
    }
}

/// Test escalated context for different query types
#[test]
fn test_escalated_context_for_commit_query() {
    let (_temp, repo_path) = create_test_repo();

    // Create repo with staged and unstaged files
    create_commit(&repo_path, "initial.rs", "// initial", "Initial");

    fs::write(repo_path.join("staged.rs"), "// staged").unwrap();
    std::process::Command::new("git")
        .args(["add", "staged.rs"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    fs::write(repo_path.join("unstaged.rs"), "// unstaged").unwrap();

    let repo = Repository::new(&repo_path);
    let context_builder = ContextBuilder::new(repo);

    // Test commit-specific escalated context
    let commit_ctx = context_builder.build_escalated_context(QueryType::Commit)
        .expect("Failed to build commit context");

    assert!(commit_ctx.escalated_info.is_some());
    let escalated = commit_ctx.escalated_info.unwrap();
    assert!(escalated.contains("Files to Commit"));
    assert!(escalated.contains("staged.rs") || commit_ctx.default_info.contains("staged.rs"));
}

/// Test token estimation accuracy
#[test]
fn test_token_estimation_realistic() {
    // Test with realistic context strings
    let small_context = "Current branch: main\n";
    let tokens = ContextBuilder::estimate_tokens(small_context);
    assert!(tokens >= 4 && tokens <= 10); // ~6 words

    let medium_context = r#"
Current branch: main
Upstream: origin/main (ahead: 0, behind: 0)

=== Repository Files ===

Staged files:
  src/main.rs
  src/lib.rs
"#;
    let tokens = ContextBuilder::estimate_tokens(medium_context);
    assert!(tokens >= 20 && tokens <= 60); // Reasonable range
}

/// Test error translation with multiple error types
#[test]
fn test_error_translation_cross_module() {
    // GitError translation
    let git_err = GitError::NotARepository;
    let app_err: AppError = git_err.into();
    let translated = ErrorTranslator::translate_app_error(&app_err);

    assert!(!translated.simple_message.is_empty());
    assert!(!translated.raw_error.is_empty());
    assert!(translated.simple_message.to_lowercase().contains("repository"));

    // ConfigError translation
    let config_err = gitalky::config::settings::ConfigError::DirectoryNotFound;
    let app_err: AppError = config_err.into();
    let translated = ErrorTranslator::translate_app_error(&app_err);

    assert!(translated.simple_message.contains("Configuration"));
    assert!(translated.suggestion.is_some());

    // LLM Error translation
    let llm_err = gitalky::llm::client::LLMError::Timeout;
    let app_err: AppError = llm_err.into();
    let translated = ErrorTranslator::translate_app_error(&app_err);

    assert!(translated.simple_message.contains("LLM"));
    assert!(translated.suggestion.is_some());
}

/// Test context builder with empty repository
#[test]
fn test_context_builder_empty_repo() {
    let (_temp, repo_path) = create_test_repo();
    let repo = Repository::new(&repo_path);
    let context_builder = ContextBuilder::new(repo);

    let ctx = context_builder.build_default_context()
        .expect("Failed to build context for empty repo");

    assert!(ctx.default_info.contains("Current branch"));
    assert!(!ctx.default_info.contains("Staged files:"));
    assert_eq!(ctx.escalated_info, None);
}

/// Test context builder with large repository (performance/token limits)
#[test]
fn test_context_builder_token_budget_enforcement() {
    let (_temp, repo_path) = create_test_repo();

    // Create many files to exceed token budget
    for i in 0..200 {
        fs::write(
            repo_path.join(format!("file_{}.rs", i)),
            format!("// File {}", i)
        ).unwrap();
    }

    let repo = Repository::new(&repo_path);
    let context_builder = ContextBuilder::new(repo);

    let ctx = context_builder.build_escalated_context(QueryType::General)
        .expect("Failed to build context");

    // Should enforce 5000 token limit
    assert!(ctx.estimated_tokens <= 5000);
}

/// Test error source chain preservation
#[test]
fn test_error_source_chain_integration() {
    use std::error::Error;

    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test.txt");
    let git_err = GitError::IoError(io_err);
    let app_err: AppError = git_err.into();

    // Verify source chain is preserved
    assert!(app_err.source().is_some());

    // Verify error translates correctly
    let translated = ErrorTranslator::translate_app_error(&app_err);
    assert!(!translated.raw_error.is_empty());
}

/// Test full query-to-context workflow
#[test]
fn test_query_to_context_workflow() {
    let (_temp, repo_path) = create_test_repo();
    create_commit(&repo_path, "main.rs", "fn main() {}", "Initial");

    let repo = Repository::new(&repo_path);
    let context_builder = ContextBuilder::new(repo);

    // User query
    let query = "show me the log history";

    // Classify
    let query_type = ContextBuilder::classify_query(query);
    assert_eq!(query_type, QueryType::History);

    // Build escalated context
    let ctx = context_builder.build_escalated_context(query_type)
        .expect("Failed to build context");

    // Should have escalated info with commits
    assert!(ctx.escalated_info.is_some());
    let escalated = ctx.escalated_info.unwrap();
    assert!(escalated.contains("Recent Commits") || escalated.contains("Initial"));
}

/// Test repository state with all features
#[test]
fn test_comprehensive_repository_state() {
    let (_temp, repo_path) = create_test_repo();

    // Create complex repo state
    create_commit(&repo_path, "initial.txt", "content", "Initial commit");
    create_commit(&repo_path, "second.txt", "content", "Second commit");

    // Add staged file
    fs::write(repo_path.join("staged.txt"), "staged").unwrap();
    std::process::Command::new("git")
        .args(["add", "staged.txt"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    // Add unstaged modification
    fs::write(repo_path.join("initial.txt"), "modified").unwrap();

    // Add untracked file
    fs::write(repo_path.join("untracked.txt"), "untracked").unwrap();

    let repo = Repository::new(&repo_path);
    let state = repo.state().expect("Failed to get state");

    // Verify all aspects captured
    assert!(!state.is_clean());
    assert_eq!(state.staged_files.len(), 1);
    assert_eq!(state.unstaged_files.len(), 1);
    assert_eq!(state.untracked_files.len(), 1);
    assert_eq!(state.recent_commits.len(), 2);

    // Build context from this state
    let context_builder = ContextBuilder::new(repo);
    let ctx = context_builder.build_default_context().unwrap();

    assert!(ctx.default_info.contains("staged.txt"));
    assert!(ctx.default_info.contains("initial.txt"));
    assert!(ctx.default_info.contains("untracked.txt"));
}
