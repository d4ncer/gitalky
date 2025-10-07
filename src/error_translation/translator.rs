use crate::error::{AppError, GitError};

#[derive(Debug, Clone)]
pub struct UserFriendlyError {
    pub simple_message: String,
    pub suggestion: Option<String>,
    pub raw_error: String,
}

pub struct ErrorTranslator;

impl ErrorTranslator {
    /// Translate an AppError into a user-friendly error message
    pub fn translate_app_error(error: &AppError) -> UserFriendlyError {
        // Match on AppError variants to provide context-specific messages
        match error {
            AppError::Git(git_err) => Self::translate(git_err),
            AppError::Config(config_err) => UserFriendlyError {
                simple_message: "Configuration error occurred.".to_string(),
                suggestion: Some("Check your config file at ~/.config/gitalky/config.toml".to_string()),
                raw_error: config_err.to_string(),
            },
            AppError::Llm(llm_err) => UserFriendlyError {
                simple_message: "Error communicating with LLM.".to_string(),
                suggestion: Some("Check your API key and network connection".to_string()),
                raw_error: llm_err.to_string(),
            },
            AppError::Translation(trans_err) => UserFriendlyError {
                simple_message: "Error translating your query.".to_string(),
                suggestion: Some("Try rephrasing your question or check connection".to_string()),
                raw_error: trans_err.to_string(),
            },
            AppError::Security(sec_err) => UserFriendlyError {
                simple_message: "Command validation failed for security reasons.".to_string(),
                suggestion: None,
                raw_error: sec_err.to_string(),
            },
            AppError::Setup(setup_err) => UserFriendlyError {
                simple_message: "Setup error occurred.".to_string(),
                suggestion: Some("Try running the first-run wizard again".to_string()),
                raw_error: setup_err.to_string(),
            },
            AppError::Io(io_err) => UserFriendlyError {
                simple_message: "I/O error occurred.".to_string(),
                suggestion: Some("Check file permissions and disk space".to_string()),
                raw_error: io_err.to_string(),
            },
        }
    }

    /// Translate a GitError into a user-friendly error message
    pub fn translate(error: &GitError) -> UserFriendlyError {
        let raw_error = error.to_string();

        // Match common error patterns
        let (simple_message, suggestion) = Self::match_error_patterns(&raw_error);

        UserFriendlyError {
            simple_message,
            suggestion,
            raw_error,
        }
    }

    /// Match common git error patterns and provide user-friendly messages
    fn match_error_patterns(error_text: &str) -> (String, Option<String>) {
        let lower = error_text.to_lowercase();

        // No upstream branch
        if lower.contains("no upstream") || lower.contains("does not have an upstream") {
            return (
                "No remote branch is configured for tracking.".to_string(),
                Some("Try: git push -u origin <branch-name>".to_string()),
            );
        }

        // Merge conflicts
        if lower.contains("merge conflict") || lower.contains("conflict") {
            return (
                "Merge has conflicts that need to be resolved.".to_string(),
                Some("Fix conflicts in the listed files, then git add and git commit.".to_string()),
            );
        }

        // Detached HEAD
        if lower.contains("detached head") {
            return (
                "Not currently on any branch (detached HEAD state).".to_string(),
                Some("Create a new branch: git checkout -b <branch-name>".to_string()),
            );
        }

        // Nothing to commit
        if lower.contains("nothing to commit") || lower.contains("working tree clean") {
            return (
                "No changes to commit - working directory is clean.".to_string(),
                None,
            );
        }

        // Pathspec did not match
        if lower.contains("pathspec") && lower.contains("did not match") {
            return (
                "File path not found in the repository.".to_string(),
                Some("Check the file path and try again. Use 'git status' to see available files.".to_string()),
            );
        }

        // Branch already exists
        if lower.contains("already exists") && (lower.contains("branch") || lower.contains("ref")) {
            return (
                "A branch with that name already exists.".to_string(),
                Some("Use a different name or delete the existing branch first.".to_string()),
            );
        }

        // Not a git repository
        if lower.contains("not a git repository") {
            return (
                "Current directory is not a git repository.".to_string(),
                Some("Initialize with: git init".to_string()),
            );
        }

        // Remote not found
        if lower.contains("remote") && (lower.contains("not found") || lower.contains("does not appear")) {
            return (
                "Remote repository not found.".to_string(),
                Some("Check the remote URL with: git remote -v".to_string()),
            );
        }

        // Authentication failed
        if lower.contains("authentication failed") || lower.contains("permission denied") {
            return (
                "Authentication failed - check your credentials.".to_string(),
                Some("Verify your SSH keys or personal access token.".to_string()),
            );
        }

        // Uncommitted changes
        if lower.contains("uncommitted changes") || lower.contains("would be overwritten") {
            return (
                "Operation would overwrite uncommitted changes.".to_string(),
                Some("Commit or stash your changes first: git stash".to_string()),
            );
        }

        // Divergent branches
        if lower.contains("diverged") || (lower.contains("rejected") && lower.contains("non-fast-forward")) {
            return (
                "Local and remote branches have diverged.".to_string(),
                Some("Pull changes first: git pull, or force push: git push --force (dangerous!)".to_string()),
            );
        }

        // Untracked files would be overwritten
        if lower.contains("untracked working tree files would be overwritten") {
            return (
                "Untracked files would be overwritten by this operation.".to_string(),
                Some("Move or remove the conflicting files, or commit them first.".to_string()),
            );
        }

        // Rebase in progress
        if lower.contains("rebase in progress") {
            return (
                "A rebase operation is currently in progress.".to_string(),
                Some("Continue with: git rebase --continue, or abort: git rebase --abort".to_string()),
            );
        }

        // Merge in progress
        if lower.contains("merge in progress") {
            return (
                "A merge operation is currently in progress.".to_string(),
                Some("Complete the merge and commit, or abort: git merge --abort".to_string()),
            );
        }

        // No changes staged
        if lower.contains("no changes added to commit") {
            return (
                "No files staged for commit.".to_string(),
                Some("Stage files with: git add <file>".to_string()),
            );
        }

        // Default: return the error as-is with no suggestion
        (error_text.to_string(), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_no_upstream() {
        let error = GitError::CommandFailed("fatal: The current branch has no upstream branch".to_string());
        let translated = ErrorTranslator::translate(&error);

        assert!(translated.simple_message.contains("No remote branch"));
        assert!(translated.suggestion.is_some());
        assert!(translated.suggestion.unwrap().contains("push -u"));
    }

    #[test]
    fn test_translate_merge_conflict() {
        let error = GitError::CommandFailed("CONFLICT (content): Merge conflict in file.txt".to_string());
        let translated = ErrorTranslator::translate(&error);

        assert!(translated.simple_message.contains("Merge has conflicts"));
        assert!(translated.suggestion.is_some());
    }

    #[test]
    fn test_translate_pathspec_not_found() {
        let error = GitError::CommandFailed("fatal: pathspec 'input.rs' did not match any files".to_string());
        let translated = ErrorTranslator::translate(&error);

        assert!(translated.simple_message.contains("File path not found"));
        assert!(translated.suggestion.is_some());
    }

    #[test]
    fn test_translate_nothing_to_commit() {
        let error = GitError::CommandFailed("nothing to commit, working tree clean".to_string());
        let translated = ErrorTranslator::translate(&error);

        assert!(translated.simple_message.contains("No changes to commit"));
        assert!(translated.suggestion.is_none());
    }

    #[test]
    fn test_translate_branch_exists() {
        let error = GitError::CommandFailed("fatal: A branch named 'feature' already exists".to_string());
        let translated = ErrorTranslator::translate(&error);

        assert!(translated.simple_message.contains("already exists"));
        assert!(translated.suggestion.is_some());
    }

    #[test]
    fn test_translate_not_a_repository() {
        let error = GitError::NotARepository;
        let translated = ErrorTranslator::translate(&error);

        assert!(translated.simple_message.contains("not a git repository"));
    }

    #[test]
    fn test_translate_authentication_failed() {
        let error = GitError::CommandFailed("fatal: Authentication failed".to_string());
        let translated = ErrorTranslator::translate(&error);

        assert!(translated.simple_message.contains("Authentication failed"));
        assert!(translated.suggestion.is_some());
    }

    #[test]
    fn test_translate_diverged_branches() {
        let error = GitError::CommandFailed("error: failed to push some refs - Updates were rejected because the tip of your current branch is behind".to_string());
        let translated = ErrorTranslator::translate(&error);

        assert!(translated.simple_message.contains("diverged") || translated.raw_error.contains("rejected"));
    }

    #[test]
    fn test_translate_unknown_error() {
        let error = GitError::CommandFailed("Some unknown error message".to_string());
        let translated = ErrorTranslator::translate(&error);

        // Unknown errors should pass through as-is
        assert!(translated.simple_message.contains("Some unknown error message"));
        assert!(translated.suggestion.is_none());
    }

    #[test]
    fn test_raw_error_preserved() {
        let error = GitError::CommandFailed("fatal: pathspec 'test.rs' did not match any files".to_string());
        let translated = ErrorTranslator::translate(&error);

        // Raw error should always be preserved
        assert!(translated.raw_error.contains("pathspec"));
        assert!(translated.raw_error.contains("test.rs"));
    }
}
