use crate::error::Result;
use crate::git::Repository;

/// Types of queries that require different context
#[derive(Debug, Clone, PartialEq)]
pub enum QueryType {
    Commit,
    Branch,
    Diff,
    History,
    Stash,
    General,
}

/// Repository context for LLM with token budget tracking
#[derive(Debug, Clone)]
pub struct RepoContext {
    pub default_info: String,
    pub escalated_info: Option<String>,
    pub estimated_tokens: usize,
}

/// Builds context for LLM queries with token budget enforcement
pub struct ContextBuilder {
    repo: Repository,
}

impl ContextBuilder {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }

    /// Build default context (~500 tokens)
    pub fn build_default_context(&self) -> Result<RepoContext> {
        let state = self.repo.state()?;
        let mut context = String::new();

        // Current branch and upstream
        if let Some(ref branch) = state.current_branch {
            context.push_str(&format!("Current branch: {}\n", branch));

            if let Some(ref upstream) = state.upstream {
                context.push_str(&format!(
                    "Upstream: {} (ahead: {}, behind: {})\n",
                    upstream.remote_branch, upstream.ahead, upstream.behind
                ));
            }
        } else {
            context.push_str("Detached HEAD state\n");
        }

        // File lists with paths - critical for fuzzy matching
        context.push_str("\n=== Repository Files ===\n");

        if !state.staged_files.is_empty() {
            context.push_str("\nStaged files:\n");
            for file in state.staged_files.iter().take(50) {
                context.push_str(&format!("  {}\n", file.path));
            }
        }

        if !state.unstaged_files.is_empty() {
            context.push_str("\nUnstaged files:\n");
            for file in state.unstaged_files.iter().take(50) {
                context.push_str(&format!("  {}\n", file.path));
            }
        }

        if !state.untracked_files.is_empty() {
            context.push_str("\nUntracked files:\n");
            for file in state.untracked_files.iter().take(50) {
                context.push_str(&format!("  {}\n", file.path));
            }
        }

        // Recent commits (just count for default)
        context.push_str(&format!("\nRecent commits: {}\n", state.recent_commits.len()));

        // Stashes
        if !state.stashes.is_empty() {
            context.push_str(&format!("Stashes: {}\n", state.stashes.len()));
        }

        // Special states
        if state.in_merge {
            context.push_str("\nMerge in progress\n");
        }
        if state.in_rebase {
            context.push_str("\nRebase in progress\n");
        }

        let estimated_tokens = Self::estimate_tokens(&context);

        Ok(RepoContext {
            default_info: context,
            escalated_info: None,
            estimated_tokens,
        })
    }

    /// Build escalated context based on query type
    pub fn build_escalated_context(&self, query_type: QueryType) -> Result<RepoContext> {
        let mut ctx = self.build_default_context()?;
        let state = self.repo.state()?;

        let escalated = match query_type {
            QueryType::Commit => {
                // Add staged/unstaged file details
                let mut info = String::from("\n=== Files to Commit ===\n");

                if !state.staged_files.is_empty() {
                    info.push_str("Staged:\n");
                    for file in state.staged_files.iter().take(20) {
                        info.push_str(&format!("  {:?}: {}\n", file.status, file.path));
                    }
                }

                if !state.unstaged_files.is_empty() {
                    info.push_str("\nUnstaged:\n");
                    for file in state.unstaged_files.iter().take(20) {
                        info.push_str(&format!("  {:?}: {}\n", file.status, file.path));
                    }
                }

                Some(info)
            }

            QueryType::Branch => {
                // Add upstream and branch info
                let mut info = String::from("\n=== Branch Info ===\n");
                if let Some(ref upstream) = state.upstream {
                    info.push_str(&format!("Tracking: {}\n", upstream.remote_branch));
                    info.push_str(&format!("Status: {} ahead, {} behind\n", upstream.ahead, upstream.behind));
                }
                Some(info)
            }

            QueryType::History => {
                // Add recent commit details
                let mut info = String::from("\n=== Recent Commits ===\n");
                for commit in state.recent_commits.iter().take(10) {
                    info.push_str(&format!("{}: {}\n", &commit.hash[..7], commit.message));
                }
                Some(info)
            }

            QueryType::Stash => {
                // Add stash details
                if !state.stashes.is_empty() {
                    let mut info = String::from("\n=== Stashes ===\n");
                    for stash in &state.stashes {
                        info.push_str(&format!("{}: {}\n", stash.index, stash.message));
                    }
                    Some(info)
                } else {
                    None
                }
            }

            QueryType::Diff => {
                // Add file change details
                let mut info = String::from("\n=== Changes ===\n");
                for file in state.unstaged_files.iter().take(15) {
                    info.push_str(&format!("{:?}: {}\n", file.status, file.path));
                }
                Some(info)
            }

            QueryType::General => None,
        };

        if let Some(ref escalated_info) = escalated {
            ctx.escalated_info = Some(escalated_info.clone());
            ctx.estimated_tokens = Self::estimate_tokens(&ctx.get_full_context());
        }

        // Enforce token budget
        if ctx.estimated_tokens > 5000 {
            self.truncate_to_budget(&mut ctx, 5000);
        }

        Ok(ctx)
    }

    /// Classify query based on keywords
    pub fn classify_query(query: &str) -> QueryType {
        let query_lower = query.to_lowercase();

        if query_lower.contains("commit") || query_lower.contains("stage") {
            QueryType::Commit
        } else if query_lower.contains("branch") || query_lower.contains("checkout") {
            QueryType::Branch
        } else if query_lower.contains("diff") || query_lower.contains("change") {
            QueryType::Diff
        } else if query_lower.contains("log") || query_lower.contains("history") {
            QueryType::History
        } else if query_lower.contains("stash") {
            QueryType::Stash
        } else {
            QueryType::General
        }
    }

    /// Estimate tokens using 4 characters ≈ 1 token heuristic
    pub fn estimate_tokens(text: &str) -> usize {
        text.len().div_ceil(4)
    }

    /// Truncate context to fit within token budget
    fn truncate_to_budget(&self, context: &mut RepoContext, max_tokens: usize) {
        if context.estimated_tokens <= max_tokens {
            return;
        }

        eprintln!(
            "Warning: Context exceeds token budget ({} > {}), truncating...",
            context.estimated_tokens, max_tokens
        );

        // Strategy: Keep default info, truncate escalated info
        if let Some(ref mut escalated) = context.escalated_info {
            let default_tokens = Self::estimate_tokens(&context.default_info);
            let available_for_escalated = max_tokens.saturating_sub(default_tokens);

            if available_for_escalated > 0 {
                let max_chars = available_for_escalated * 4;
                if escalated.len() > max_chars {
                    escalated.truncate(max_chars);
                    escalated.push_str("\n... [truncated]");
                }
            } else {
                // No room for escalated info
                context.escalated_info = None;
            }
        }

        // Recalculate tokens
        context.estimated_tokens = Self::estimate_tokens(&context.get_full_context());
    }
}

impl RepoContext {
    /// Get full context string (default + escalated)
    pub fn get_full_context(&self) -> String {
        let mut full = self.default_info.clone();
        if let Some(ref escalated) = self.escalated_info {
            full.push_str(escalated);
        }
        full
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_classification() {
        assert_eq!(ContextBuilder::classify_query("commit all changes"), QueryType::Commit);
        assert_eq!(ContextBuilder::classify_query("create a new branch"), QueryType::Branch);
        assert_eq!(ContextBuilder::classify_query("show me the diff"), QueryType::Diff);
        assert_eq!(ContextBuilder::classify_query("view log history"), QueryType::History);
        assert_eq!(ContextBuilder::classify_query("stash my work"), QueryType::Stash);
        assert_eq!(ContextBuilder::classify_query("what's the status?"), QueryType::General);
    }

    #[test]
    fn test_token_estimation() {
        assert_eq!(ContextBuilder::estimate_tokens("test"), 1);
        assert_eq!(ContextBuilder::estimate_tokens("12345678"), 2);
        assert_eq!(ContextBuilder::estimate_tokens("1234567890123456"), 4);
    }

    #[test]
    fn test_context_get_full() {
        let ctx = RepoContext {
            default_info: "default".to_string(),
            escalated_info: Some("escalated".to_string()),
            estimated_tokens: 5,
        };

        assert_eq!(ctx.get_full_context(), "defaultescalated");
    }
}
