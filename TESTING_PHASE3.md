# Phase 3 Manual Testing Guide

This guide helps you manually test the LLM integration implemented in Phase 3.

## Prerequisites

1. **Anthropic API Key**: You need a Claude API key from https://console.anthropic.com/
2. **Git repository**: Run tests from within this repository or any git repository

## Testing Options

### Option 1: Inspect Context (No API Key Required)

This shows you what context would be sent to the LLM without actually calling the API.

```bash
# See default context
cargo run --example inspect_context

# See context for different query types
cargo run --example inspect_context "commit my changes"
cargo run --example inspect_context "show me the diff"
cargo run --example inspect_context "create a new branch"
cargo run --example inspect_context "show me the history"
cargo run --example inspect_context "list my stashes"
```

**What this tests:**
- Query classification (Commit, Branch, Diff, History, Stash, General)
- Default context building (~500 tokens)
- Escalated context building (query-specific details)
- Token budget enforcement (5000 token limit)

### Option 2: Full LLM Integration (Requires API Key)

This actually calls the Claude API and translates natural language to git commands.

```bash
# Set your API key
export ANTHROPIC_API_KEY="sk-ant-..."

# Test various queries
cargo run --example test_llm "show me what changed"
cargo run --example test_llm "list all branches"
cargo run --example test_llm "show me the last 5 commits"
cargo run --example test_llm "create a new branch called feature-xyz"
cargo run --example test_llm "stage all my changes"
cargo run --example test_llm "show me the difference between branches"
```

**What this tests:**
- Full translation pipeline
- Anthropic API integration
- Retry logic and rate limiting
- Response parsing and validation
- Error handling

## Test Scenarios

### Scenario 1: Basic Status Query
```bash
cargo run --example test_llm "show me the status"
# Expected: git status
```

### Scenario 2: Query with Repository Context
```bash
# First make some changes
echo "test" > test.txt
git add test.txt

cargo run --example test_llm "commit my staged changes"
# Expected: git commit (possibly with -m flag)
```

### Scenario 3: Query Classification
```bash
# Test different query types
cargo run --example inspect_context "commit all my work"
# Should classify as: QueryType::Commit

cargo run --example inspect_context "show me the changes"
# Should classify as: QueryType::Diff

cargo run --example inspect_context "switch to main branch"
# Should classify as: QueryType::Branch
```

### Scenario 4: Context Escalation
```bash
# Create multiple staged and unstaged files
echo "staged1" > file1.txt
echo "staged2" > file2.txt
git add file1.txt file2.txt
echo "unstaged" > file3.txt

# Inspect commit context (should show file details)
cargo run --example inspect_context "commit my changes"
# Should show escalated context with staged/unstaged file lists
```

### Scenario 5: Token Budget Enforcement
```bash
# In a repository with many commits
cargo run --example inspect_context "show me the history"
# Check that estimated tokens doesn't exceed 5000
# Should show "[truncated]" if context was too large
```

## What to Verify

### ✅ Context Builder
- [ ] Query classification works for all types (Commit, Branch, Diff, History, Stash, General)
- [ ] Default context includes: branch, upstream, file counts, commit count, stashes, merge/rebase state
- [ ] Escalated context adds relevant details based on query type
- [ ] Token estimation is reasonable (text length / 4)
- [ ] Context truncates to 5000 tokens when needed

### ✅ Anthropic Client
- [ ] API key is read from environment
- [ ] API calls succeed with valid key
- [ ] Responses are parsed correctly
- [ ] Git commands are validated (start with "git" or are git subcommands)
- [ ] Error messages are helpful

### ✅ Translator
- [ ] Combines context builder and LLM client correctly
- [ ] Async translation works
- [ ] Returns GitCommand struct with command string

## Error Testing

### Test 1: Missing API Key
```bash
unset ANTHROPIC_API_KEY
cargo run --example test_llm "show me the status"
# Expected: Clear error message about missing API key
```

### Test 2: Invalid API Key
```bash
export ANTHROPIC_API_KEY="invalid-key"
cargo run --example test_llm "show me the status"
# Expected: API error from Anthropic
```

### Test 3: Not in Git Repository
```bash
cd /tmp
cargo run --example test_llm "show me the status"
# Expected: Error about not being in a git repository
```

### Test 4: Rate Limiting (if you hit limits)
```bash
# Run many requests quickly
for i in {1..10}; do
  cargo run --example test_llm "show me the status"
done
# Expected: Retry with exponential backoff, eventual success or clear error
```

## Expected Output Examples

### inspect_context example:
```
🔍 Repository: /Users/rk/code/gitalky
❓ Query: show me the status

🏷️  Query Type: General

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
DEFAULT CONTEXT (~500 tokens):
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Current branch: main
Upstream: origin/main (ahead: 0, behind: 0)

Staged files: 0
Unstaged files: 1
Untracked files: 2

Recent commits: 3

📊 Estimated tokens: 42
...
```

### test_llm example:
```
🔍 Repository: /Users/rk/code/gitalky
❓ Query: show me what changed

⏳ Translating with Claude...

✅ Success!
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Git Command: git status
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

💡 You can now run this command manually
```

## Troubleshooting

**Problem**: "Error: Not in a git repository"
- **Solution**: Run from within the gitalky directory or any git repository

**Problem**: "Error: ANTHROPIC_API_KEY environment variable not set"
- **Solution**: `export ANTHROPIC_API_KEY="your-key"`

**Problem**: API timeout or network errors
- **Solution**: Check internet connection, verify API key is valid

**Problem**: "Response doesn't look like a git command"
- **Solution**: This is a validation error - the LLM returned something unexpected. Try a different query or check the prompt in `src/llm/anthropic.rs`

## Cost Considerations

Each API call to Claude costs approximately:
- Input tokens: ~500-5000 tokens (context)
- Output tokens: ~20-100 tokens (git command)
- Model: claude-sonnet-4-5-20250929

Check pricing at: https://www.anthropic.com/pricing

For testing, a few dozen queries should cost less than $0.10 total.

## Next Steps

After verifying Phase 3 works:
- Phase 4 will integrate this into the TUI
- You'll be able to press a key in the TUI to enter natural language queries
- The git command will be shown for review before execution
