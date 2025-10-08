mod helpers;

use gitalky::error::{AppError, GitError};
use gitalky::error_translation::translator::ErrorTranslator;
use gitalky::git::parser::*;
use gitalky::llm::context::ContextBuilder;
use helpers::{create_commit, create_test_repo};
use std::fs;

/// Test parsing completely empty git output
#[test]
fn test_parse_empty_outputs() {
    assert_eq!(parse_status_porcelain_v2("").unwrap().len(), 0);
    assert_eq!(parse_log("").unwrap().len(), 0);
    assert_eq!(parse_branch_list("").unwrap().len(), 0);
    assert_eq!(parse_stash_list("").unwrap().len(), 0);
}

/// Test parsing malformed git status output
#[test]
fn test_parse_malformed_status() {
    // Missing fields
    let malformed = "1 M.";
    let result = parse_status_porcelain_v2(malformed).unwrap();
    assert_eq!(result.len(), 0); // Should skip malformed lines

    // Invalid status codes
    let invalid = "1 XX N... 100644 100644 100644 abc123 def456 file.txt";
    let result = parse_status_porcelain_v2(invalid).unwrap();
    assert_eq!(result.len(), 1); // Should still parse but mark as Unknown
}

/// Test parsing file paths with spaces
#[test]
fn test_parse_paths_with_spaces() {
    let output = "1 M. N... 100644 100644 100644 abc123 def456 my file with spaces.txt";
    let result = parse_status_porcelain_v2(output).unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].path, "my file with spaces.txt");
}

/// Test parsing very long file paths
#[test]
fn test_parse_very_long_paths() {
    let long_path = "a/".repeat(100) + "file.txt";
    let output = format!("1 M. N... 100644 100644 100644 abc123 def456 {}", long_path);
    let result = parse_status_porcelain_v2(&output).unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].path, long_path);
}

/// Test parsing commits with null bytes
#[test]
fn test_parse_commits_with_nulls() {
    let output = "abc123\0Message with\0embedded\0nulls";
    let result = parse_log(output).unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].hash, "abc123");
    // Only first null byte is the delimiter
    assert_eq!(result[0].message, "Message with");
}

/// Test parsing commits with empty messages
#[test]
fn test_parse_commits_empty_messages() {
    let output = "abc123\0\ndef456\0";
    let result = parse_log(output).unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].message, "");
    assert_eq!(result[1].message, "");
}

/// Test parsing commits with very long messages
#[test]
fn test_parse_commits_long_messages() {
    let long_msg = "A".repeat(10000);
    let output = format!("abc123\0{}", long_msg);
    let result = parse_log(&output).unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].message.len(), 10000);
}

/// Test parsing branch names with special characters
#[test]
fn test_parse_branches_special_chars() {
    let output = "* feature/FOO-123\n  bugfix/issue-456\n  release/v1.0.0";
    let result = parse_branch_list(output).unwrap();

    assert_eq!(result.len(), 3);
    assert_eq!(result[0].name, "feature/FOO-123");
    assert!(result[0].is_current);
    assert_eq!(result[1].name, "bugfix/issue-456");
    assert_eq!(result[2].name, "release/v1.0.0");
}

/// Test error translation with edge case messages
#[test]
fn test_error_translation_edge_cases() {
    // Empty error message
    let err = GitError::CommandFailed("".to_string());
    let translated = ErrorTranslator::translate(&err);
    assert!(!translated.simple_message.is_empty());

    // Very long error message
    let long_msg = "error: ".to_string() + &"x".repeat(5000);
    let err = GitError::CommandFailed(long_msg);
    let translated = ErrorTranslator::translate(&err);
    assert!(!translated.simple_message.is_empty());

    // Error with unicode characters
    let err = GitError::CommandFailed("fatal: 文件不存在 файл не найден".to_string());
    let translated = ErrorTranslator::translate(&err);
    assert!(!translated.simple_message.is_empty());
}

/// Test error translation with mixed case patterns
#[test]
fn test_error_translation_case_insensitive() {
    let patterns = vec![
        "FATAL: NOT A GIT REPOSITORY",
        "Fatal: Not a git Repository",
        "fatal: not a git repository",
    ];

    for pattern in patterns {
        let err = GitError::CommandFailed(pattern.to_string());
        let translated = ErrorTranslator::translate(&err);
        assert!(
            translated.simple_message.contains("not a git repository"),
            "Pattern '{}' should be recognized",
            pattern
        );
    }
}

/// Test context builder with maximum file count
#[test]
fn test_context_builder_max_files() {
    let (_temp, repo_path) = create_test_repo();

    // Create many files (> 50 which is the display limit)
    for i in 0..100 {
        fs::write(repo_path.join(format!("file{}.txt", i)), "content").unwrap();
    }

    let repo = gitalky::git::Repository::new(&repo_path);
    let context_builder = ContextBuilder::new(repo);

    let ctx = context_builder.build_default_context().unwrap();

    // Should only include first 50 files per category (untracked in this case)
    // The context limits display to 50 files per category
    let untracked_section = ctx.default_info.contains("Untracked files:");
    assert!(untracked_section);

    // Count files in untracked section (they're listed as "  filename")
    let untracked_count = ctx.default_info.lines()
        .filter(|line| line.starts_with("  file"))
        .count();
    assert!(untracked_count <= 50, "Should limit to 50 files, got {}", untracked_count);
}

/// Test context builder with unicode file names
#[test]
fn test_context_builder_unicode_files() {
    let (_temp, repo_path) = create_test_repo();

    fs::write(repo_path.join("文件.txt"), "content").unwrap();
    fs::write(repo_path.join("файл.rs"), "content").unwrap();
    fs::write(repo_path.join("αρχείο.md"), "content").unwrap();

    let repo = gitalky::git::Repository::new(&repo_path);
    let context_builder = ContextBuilder::new(repo);

    let ctx = context_builder.build_default_context().unwrap();

    // Should handle unicode filenames
    assert!(ctx.default_info.contains("文件.txt") ||
            ctx.default_info.contains("Untracked files"));
}

/// Test token estimation edge cases
#[test]
fn test_token_estimation_edge_cases() {
    // Empty string
    assert_eq!(ContextBuilder::estimate_tokens(""), 0);

    // Single character
    assert_eq!(ContextBuilder::estimate_tokens("a"), 1);

    // Exactly 4 characters (should round up)
    assert_eq!(ContextBuilder::estimate_tokens("1234"), 1);

    // Unicode characters (counted by bytes in UTF-8)
    let unicode = "你好世界"; // 4 Chinese characters = 12 bytes in UTF-8
    let tokens = ContextBuilder::estimate_tokens(unicode);
    assert!(tokens >= 3); // 12 bytes / 4 = 3 tokens
}

/// Test repository state with no .git directory
#[test]
fn test_repository_discover_no_git() {
    let temp_dir = tempfile::TempDir::new().unwrap();

    let result = gitalky::git::Repository::discover_from(temp_dir.path());

    assert!(result.is_err());
    match result.unwrap_err() {
        GitError::NotARepository => {}, // Expected
        other => panic!("Expected NotARepository, got {:?}", other),
    }
}

/// Test repository state with corrupted .git directory
#[test]
fn test_repository_corrupted_git_dir() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let git_dir = temp_dir.path().join(".git");

    // Create empty .git file instead of directory
    fs::write(&git_dir, "").unwrap();

    let repo = gitalky::git::Repository::new(temp_dir.path());

    // Getting state should handle this gracefully
    let result = repo.state();
    assert!(result.is_err()); // Should error, not panic
}

/// Test error conversion with all AppError variants
#[test]
fn test_all_app_error_variants() {
    let errors: Vec<AppError> = vec![
        GitError::NotARepository.into(),
        gitalky::config::settings::ConfigError::DirectoryNotFound.into(),
        gitalky::llm::client::LLMError::Timeout.into(),
        gitalky::llm::translator::TranslationError::LLMError(
            gitalky::llm::client::LLMError::Timeout
        ).into(),
        gitalky::security::validator::ValidationError::InvalidFormat.into(),
        gitalky::config::first_run::SetupError::Cancelled.into(),
        std::io::Error::new(std::io::ErrorKind::NotFound, "test").into(),
    ];

    for error in errors {
        let translated = ErrorTranslator::translate_app_error(&error);
        assert!(!translated.simple_message.is_empty());
        assert!(!translated.raw_error.is_empty());
        // Some may not have suggestions, which is fine
    }
}

/// Test query classification with empty/whitespace strings
#[test]
fn test_query_classification_edge_cases() {
    use gitalky::llm::context::QueryType;

    assert_eq!(ContextBuilder::classify_query(""), QueryType::General);
    assert_eq!(ContextBuilder::classify_query("   "), QueryType::General);
    assert_eq!(ContextBuilder::classify_query("\n\t"), QueryType::General);

    // Very long query
    let long_query = "commit ".to_string() + &"x".repeat(1000);
    assert_eq!(ContextBuilder::classify_query(&long_query), QueryType::Commit);
}

/// Test context truncation with very large escalated info
#[test]
fn test_context_truncation() {
    let (_temp, repo_path) = create_test_repo();

    // Create many commits to generate large escalated context
    for i in 0..100 {
        create_commit(
            &repo_path,
            &format!("file{}.txt", i),
            &format!("content {}", i),
            &format!("Commit number {} with a moderately long commit message", i)
        );
    }

    let repo = gitalky::git::Repository::new(&repo_path);
    let context_builder = ContextBuilder::new(repo);

    let ctx = context_builder.build_escalated_context(
        gitalky::llm::context::QueryType::History
    ).unwrap();

    // Should respect 5000 token budget
    assert!(ctx.estimated_tokens <= 5000);
}

/// Test error source chain with deeply nested errors
#[test]
fn test_deep_error_chain() {
    use std::error::Error;

    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
    let git_err = GitError::IoError(io_err);
    let app_err: AppError = git_err.into();

    // Traverse error chain
    let mut count = 0;
    let mut current: Option<&dyn Error> = Some(&app_err);
    while let Some(err) = current {
        count += 1;
        current = err.source();
    }

    assert!(count >= 2); // At least AppError -> GitError (IoError may or may not have source)
}
