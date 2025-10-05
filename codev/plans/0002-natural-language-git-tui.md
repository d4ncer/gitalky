# Plan: Natural Language Git Terminal UI

## Metadata
- **ID**: 0002-natural-language-git-tui
- **Status**: draft
- **Specification**: [codev/specs/0002-natural-language-git-tui.md](../specs/0002-natural-language-git-tui.md)
- **Created**: 2025-10-04

## Executive Summary

This plan implements a Rust-based terminal UI application (gitalky) that bridges natural language with git operations. The implementation follows **Approach 1** from the specification: Ratatui-based TUI with direct LLM integration, shelling out to git commands for maximum compatibility.

The implementation is divided into 6 phases, each delivering independently valuable functionality:
1. **Foundation & Git Integration** - Core infrastructure and git operations
2. **TUI Framework & Repository Display** - Visual interface for repository state
3. **LLM Integration & Translation** - Natural language to command translation
4. **Command Confirmation & Execution** - User review and command execution flow
5. **Error Handling & Safety** - Error translation, validation, and dangerous operation protection
6. **Configuration & First-Run Experience** - Config system, offline mode, setup flow

Each phase ends with comprehensive testing, self-review, user evaluation, and a git commit before proceeding to the next phase.

## Success Metrics

From specification:
- [ ] TUI launches and auto-detects git repository from current directory
- [ ] Repository state panel displays: current branch, status, staged/unstaged changes, untracked files, recent commits
- [ ] User can enter natural language queries in input area
- [ ] System translates natural language to git commands via LLM
- [ ] Proposed git command is displayed for user review
- [ ] User can confirm, cancel, or edit the proposed command
- [ ] Approved commands execute and results are shown
- [ ] Repository state panel automatically refreshes after operations
- [ ] Dangerous operations (force push, hard reset, etc.) require additional confirmation
- [ ] Git errors are translated to plain language with option to view raw error
- [ ] Configuration file allows setting LLM provider, model, and preferences
- [ ] All tests pass with >80% coverage
- [ ] Documentation includes setup guide and example queries

Implementation-specific metrics:
- [ ] Test coverage >85%
- [ ] UI refresh <100ms for 1000-file repositories
- [ ] Startup time <500ms (warm start)
- [ ] Memory usage <100MB during typical operation
- [ ] Zero command injection vulnerabilities in security review
- [ ] LLM command translation accuracy >90% for common operations
- [ ] All git subcommands properly validated against allowlist

## Phase Breakdown

### Phase 1: Foundation & Git Integration

**Status**: pending
**Dependencies**: None

#### Objectives
- Establish project structure and core dependencies
- Implement git command execution and output parsing
- Create repository state detection and querying
- Build foundation for safe command execution

#### Deliverables
- [ ] Cargo project with edition 2024, necessary dependencies
- [ ] `src/git/version.rs` - Git version detection and validation
- [ ] `src/git/repository.rs` - Repository detection and state queries (including stashes)
- [ ] `src/git/executor.rs` - Git command execution with proper error handling
- [ ] `src/git/parser.rs` - Parse git porcelain output (status, log, branch, stash)
- [ ] `src/error.rs` - Error types for git operations, parsing failures
- [ ] Unit tests for git integration (>85% coverage)
- [ ] Integration tests with real git repository

#### Implementation Details

**Project Setup** (`Cargo.toml`):
```toml
[package]
name = "gitalky"
version = "0.1.0"
edition = "2024"
rust-version = "1.90"

[dependencies]
# Phase 1
thiserror = "1.0"
# Phase 2 will add: ratatui, crossterm
# Phase 3 will add: tokio, reqwest, serde, serde_json, async-trait
# Phase 5 will add: toml

[dev-dependencies]
tempfile = "3.0"  # For test git repositories
```

**Git Version Detection** (`src/git/version.rs`):
- Detect installed git version via `git --version`
- Parse version string to extract major.minor version
- Validate git >= 2.20 (required for porcelain v2 format support)
- Return clear error message if version too old:
  ```
  Error: Git 2.20 or higher is required (found 2.15.0)
  Please upgrade git: https://git-scm.com/downloads
  ```

**Repository Detection** (`src/git/repository.rs`):
- Detect git repository from current working directory
- Walk up directory tree to find `.git` folder
- Query repository state:
  - Current branch name
  - Upstream tracking info
  - Ahead/behind counts
  - List of staged/unstaged/untracked files
  - Recent commits
  - Stash count and list (if any)
  - Repository state flags (merge in progress, rebase, detached HEAD)
- Use git porcelain formats for reliable parsing

**Command Executor** (`src/git/executor.rs`):
```rust
pub struct GitExecutor {
    repo_path: PathBuf,
}

impl GitExecutor {
    pub fn execute(&self, command: &str) -> Result<CommandOutput, GitError>;
    pub fn execute_with_timeout(&self, command: &str, timeout: Duration) -> Result<CommandOutput, GitError>;
}
```
- Execute git commands using `std::process::Command`
- Capture stdout, stderr, exit code
- Set working directory to repository root
- Apply timeout for long-running operations
- Sanitize inputs (basic validation, no shell interpolation)

**Output Parser** (`src/git/parser.rs`):
- Parse `git status --porcelain=v2` output
- Parse `git log --oneline --format=%H%x00%s` output
- Parse `git branch -vv --format=%(refname:short)%00%(upstream)%00%(upstream:track)`
- Parse `git stash list --format=%gd%x00%s` output
- Extract structured data from git command output
- Handle edge cases (empty repo, no commits, detached HEAD, no stashes)

**Error Handling** (`src/error.rs`):
```rust
#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("Not a git repository")]
    NotARepository,

    #[error("Git command failed: {0}")]
    CommandFailed(String),

    #[error("Failed to parse git output: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

#### Acceptance Criteria
- [ ] Detects git version and validates >= 2.20
- [ ] Shows clear error message if git version too old
- [ ] Detects git repository from any subdirectory within a repo
- [ ] Returns proper error when not in a git repository
- [ ] Executes basic git commands (status, log, branch, stash) successfully
- [ ] Parses git porcelain output correctly for all tested scenarios
- [ ] Correctly identifies and lists stashes
- [ ] Handles git errors gracefully (e.g., invalid branch name)
- [ ] All unit tests pass
- [ ] Integration tests pass with real git repository

#### Test Plan
- **Unit Tests**:
  - Repository detection logic with mocked filesystem
  - Git command builder and sanitization
  - Parser functions with sample git output
  - Error handling for various failure modes

- **Integration Tests**:
  - Create temporary git repository with known state
  - Execute status, log, branch commands
  - Verify parsed output matches expected state
  - Test edge cases: empty repo, merge conflicts, detached HEAD

- **Manual Testing**:
  - Run in various git repositories (clean, dirty, conflicted)
  - Verify output parsing accuracy

#### Rollback Strategy
This is the foundation phase - no rollback needed. If issues arise, fix them before proceeding.

#### Risks
- **Risk**: Git porcelain format parsing breaks on edge cases
  - **Mitigation**: Comprehensive test suite with various repo states; graceful error handling
- **Risk**: Git version compatibility issues (pre-2.20 doesn't have some porcelain formats)
  - **Mitigation**: Detect git version on startup via `src/git/version.rs`, provide clear error message with upgrade instructions if too old

---

### Phase 2: TUI Framework & Repository Display

**Status**: pending
**Dependencies**: Phase 1 (git integration)

#### Objectives
- Set up ratatui TUI framework with terminal backend
- Implement Magit-inspired repository state panel
- Create event loop for keyboard input
- Display live repository state

#### Deliverables
- [ ] `src/ui/app.rs` - Main application state and event loop
- [ ] `src/ui/repo_panel.rs` - Repository state display widget
- [ ] Terminal setup and teardown with crossterm
- [ ] Keyboard event handling (basic navigation, quit)
- [ ] Repository state auto-refresh on interval
- [ ] UI tests (snapshot tests for layouts)

#### Implementation Details

**Application State** (`src/ui/app.rs`):
```rust
pub struct App {
    repo: Repository,
    repo_state: RepositoryState,
    should_quit: bool,
    mode: AppMode,  // Normal, Offline
}

impl App {
    pub fn new(repo_path: PathBuf) -> Result<Self, AppError>;
    pub fn run(&mut self) -> Result<(), AppError>;
    pub fn handle_event(&mut self, event: Event) -> Result<(), AppError>;
    pub fn refresh_repo_state(&mut self) -> Result<(), AppError>;
}
```

**Repository Panel** (`src/ui/repo_panel.rs`):
- Render sections: Head, Untracked files, Unstaged changes, Staged changes, Stashes, Recent commits
- **Stash Section**:
  - Show stash count: "Stashes (3)" or hide section if zero
  - List stashes with index and description: "stash@{0}: WIP on main: fix bug"
  - Limit display to first 5 stashes if more exist
- Each section collapsible/expandable (v1: all expanded)
- Use colored text: green for added, red for deleted, yellow for modified
- Display file counts and sample entries
- Format commits with hash and message

**Layout**:
```
┌─────────────────────────────────────────────────────────────────┐
│ Gitalky - /path/to/repo                             [? for help]│
├─────────────────────────────────────────────────────────────────┤
│ Repository State                                                 │
│ ────────────────────────────────────────────────────────────────│
│ Head:     main ↑2 ↓1  (origin/main)                            │
│                                                                   │
│ Unstaged changes (2)                                            │
│   modified:   src/main.rs                                       │
│   modified:   README.md                                         │
│                                                                   │
│ Staged changes (1)                                              │
│   new file:   src/config.rs                                     │
│                                                                   │
│ Stashes (2)                                                      │
│   stash@{0}: WIP on main: experimental feature                  │
│   stash@{1}: WIP on feature-x: debugging                        │
│                                                                   │
│ Recent commits (5)                                               │
│   abc123 Add config module                                      │
│   def456 Initial commit                                         │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
Press 'q' to quit
```

**Event Loop**:
- Poll for terminal events (keyboard input)
- Handle key presses: 'q' to quit, '?' for help (future)
- Refresh repository state on interval (100ms)
- Render UI on state changes

**Terminal Management** (`src/main.rs`):
- Initialize crossterm backend
- Enter alternate screen
- Enable raw mode
- Restore terminal on exit (panic handler)

#### Acceptance Criteria
- [ ] TUI launches successfully in git repository
- [ ] Repository panel displays current branch, ahead/behind counts
- [ ] Shows staged, unstaged, and untracked files correctly
- [ ] Displays stashes if any exist, hides section if none
- [ ] Shows stash index and description correctly
- [ ] Displays recent commits with hash and message
- [ ] UI updates when repository state changes (e.g., stage a file externally, create stash)
- [ ] Pressing 'q' quits gracefully and restores terminal
- [ ] Handles terminal resize events
- [ ] Works in repositories with various states (clean, dirty, conflicts, with/without stashes)

#### Test Plan
- **Unit Tests**:
  - Repository state rendering with mock data
  - Layout calculations
  - Event handling logic

- **Integration Tests**:
  - Launch TUI with test repository
  - Verify rendered output matches expected state
  - Simulate keyboard events

- **Manual Testing**:
  - Launch in various repositories
  - Verify visual display accuracy
  - Test terminal resize behavior
  - Modify files externally and verify refresh

#### Rollback Strategy
Revert to Phase 1 by removing UI code, keeping only git integration.

#### Risks
- **Risk**: UI refresh performance with large repositories
  - **Mitigation**: Implement pagination if file count exceeds threshold (100 files); lazy loading
- **Risk**: Terminal compatibility issues
  - **Mitigation**: Test with common terminal emulators (iTerm2, Terminal.app, gnome-terminal); graceful degradation

---

### Phase 3: LLM Integration & Translation

**Status**: pending
**Dependencies**: Phase 1 (for repository context)

#### Objectives
- Implement Anthropic Claude API client
- Build natural language to git command translator
- Implement LLM context strategy with escalation rules
- Handle API errors and rate limiting

#### Deliverables
- [ ] `src/llm/client.rs` - LLM client trait and common types
- [ ] `src/llm/anthropic.rs` - Claude API implementation
- [ ] `src/llm/translator.rs` - Translation logic with context management
- [ ] `src/llm/context.rs` - Context builder with escalation rules
- [ ] API key management (from environment variable)
- [ ] Unit tests for context building
- [ ] Integration tests with real API (optional, gated by env var)

#### Implementation Details

**LLM Client Trait** (`src/llm/client.rs`):
```rust
#[async_trait]
pub trait LLMClient: Send + Sync {
    async fn translate(&self, query: &str, context: &RepoContext) -> Result<GitCommand, LLMError>;
}

pub struct GitCommand {
    pub command: String,
    pub explanation: Option<String>,
}
```

**Anthropic Client** (`src/llm/anthropic.rs`):
```rust
pub struct AnthropicClient {
    api_key: String,
    model: String,
    http_client: reqwest::Client,
}

impl AnthropicClient {
    pub fn new(api_key: String, model: String) -> Self;
    async fn call_api(&self, prompt: &str, context: &str) -> Result<String, LLMError>;
}
```
- Use `reqwest` for HTTP requests to Claude API
- Implement retry logic with exponential backoff
- Handle rate limiting (429 status code)
- Timeout after 10 seconds

**Context Builder** (`src/llm/context.rs`):
```rust
pub struct ContextBuilder {
    repo: Repository,
}

impl ContextBuilder {
    pub fn build_default_context(&self) -> Result<RepoContext, ContextError>;
    pub fn build_escalated_context(&self, query_type: QueryType) -> Result<RepoContext, ContextError>;
    pub fn classify_query(&self, query: &str) -> QueryType;
    pub fn estimate_tokens(&self, context: &RepoContext) -> usize;
    pub fn truncate_to_budget(&self, context: &mut RepoContext, max_tokens: usize);
}

pub enum QueryType {
    Commit,
    Branch,
    Diff,
    History,
    Stash,
    General,
}

pub struct RepoContext {
    pub default_info: String,      // ~500 tokens
    pub escalated_info: Option<String>,  // Additional context
    pub estimated_tokens: usize,
}
```
- Implement context escalation rules from specification
- **Token Budget Enforcement**:
  - Use simple heuristic: 4 characters ≈ 1 token (conservative estimate)
  - `estimate_tokens()`: Count characters / 4
  - `truncate_to_budget()`: Truncate oldest/least relevant info if over 5000 tokens
  - Prioritize: current state > recent changes > history
  - Log warning if truncation occurs
- Context truncation when over 5000 token cap
- Query classification heuristics (keywords: "commit", "branch", "diff", "log", "stash")
- **File Path Context** (critical for fuzzy matching):
  - Include full file paths in default context for all repository files
  - List staged, unstaged, and untracked files with complete paths
  - Limit to 50 files per category to manage token budget
  - Example format:
    ```
    === Repository Files ===

    Untracked files:
      src/ui/input.rs
      src/ui/app.rs
      ...
    ```
  - This enables LLM to fuzzy match user queries like "add input.rs" to "src/ui/input.rs"

**Translator** (`src/llm/translator.rs`):
```rust
pub struct Translator {
    client: Box<dyn LLMClient>,
    context_builder: ContextBuilder,
}

impl Translator {
    pub async fn translate(&self, query: &str) -> Result<GitCommand, TranslationError>;
}
```
- Build appropriate context based on query classification
- Construct prompt for LLM with file path matching instructions:
  ```
  You are a git command expert. Translate the user's natural language query into a git command.

  Repository Context:
  {context}

  User Query: {query}

  CRITICAL INSTRUCTIONS:
  - Respond with ONLY the git command itself
  - Do NOT include explanations, reasoning, or commentary
  - Do NOT use markdown code blocks or backticks
  - Do NOT use multiple lines
  - Output format: exactly one line containing just the git command

  FILE PATH MATCHING:
  - When the user mentions a file name, look at the repository files in the context
  - Use fuzzy matching to find the correct file path
  - If user says "add input.rs", look for files ending in "input.rs" like "src/ui/input.rs"
  - Always use the full path from the repository context
  - Prioritize exact basename matches over partial matches
  - Examples:
    * User: "add input.rs" → git add src/ui/input.rs (if that's the only input.rs)
    * User: "stage app.rs" → git add src/ui/app.rs (if that's in the file list)
    * User: "add main" → git add src/main.rs (if that's in the file list)
  ```
- Parse LLM response to extract git command
- Validate response format

#### Acceptance Criteria
- [ ] Successfully connects to Claude API with valid API key
- [ ] Translates simple queries: "show status" → `git status`
- [ ] Translates complex queries: "create branch feature-x from main" → `git checkout -b feature-x main`
- [ ] Includes appropriate context based on query type
- [ ] Handles API errors gracefully (network failure, rate limit, invalid key)
- [ ] Query classification correctly identifies commit/branch/diff/history operations
- [ ] Context stays within 5000 token budget limits
- [ ] Token estimation is reasonably accurate (±20%)
- [ ] Truncation preserves most important context
- [ ] Warning logged when context is truncated

#### Test Plan
- **Unit Tests**:
  - Context building logic with mock repository state
  - Query classification with various input queries
  - Token budget estimation with known text samples
  - Token budget truncation with oversized contexts
  - Verify truncation priority (keeps current state, drops history)
  - Command extraction from LLM responses

- **Integration Tests** (with API key):
  - Real API calls with sample queries
  - Verify translation accuracy
  - Test error handling with invalid API key

- **Manual Testing**:
  - Test 20+ common git operations via natural language
  - Verify command accuracy and appropriateness

#### Rollback Strategy
Remove LLM integration, fall back to direct git command input only (offline mode).

#### Risks
- **Risk**: LLM returns invalid or dangerous commands
  - **Mitigation**: Command validation in Phase 5; user always reviews before execution
- **Risk**: API costs exceed budget during development/testing
  - **Mitigation**: Implement request caching; use test mode with fixed responses; monitor usage
- **Risk**: Query classification is inaccurate, leading to wrong context
  - **Mitigation**: Default to general context if unsure; log misclassifications for improvement

---

### Phase 4: Command Confirmation & Execution

**Status**: pending
**Dependencies**: Phase 2 (TUI), Phase 3 (LLM)

#### Objectives
- Add input widget for natural language queries
- Display proposed commands for user review
- Implement command editing functionality
- Execute approved commands and show output
- Refresh repository state after execution

#### Deliverables
- [ ] `src/ui/input.rs` - Natural language input widget
- [ ] `src/ui/command_preview.rs` - Command confirmation widget
- [ ] `src/ui/output.rs` - Command output display
- [ ] Async runtime integration (tokio) for LLM calls
- [ ] Command execution flow with user confirmation
- [ ] Loading indicator during LLM translation
- [ ] UI tests for input and confirmation widgets

#### Implementation Details

**Input Widget** (`src/ui/input.rs`):
```rust
pub struct InputWidget {
    input: String,
    cursor_position: usize,
    mode: InputMode,  // Online, Offline
}

impl InputWidget {
    pub fn handle_key(&mut self, key: KeyEvent);
    pub fn render(&self, area: Rect, buf: &mut Buffer);
    pub fn take_input(&mut self) -> String;
}
```
- Text input with cursor
- Prompt changes based on mode: "Natural language or git command:" vs "Enter git command:"
- Submit on Enter key
- Clear input after submission

**Command Preview Widget** (`src/ui/command_preview.rs`):
```rust
pub struct CommandPreview {
    command: String,
    explanation: Option<String>,
    edit_mode: bool,
}

impl CommandPreview {
    pub fn new(command: String, explanation: Option<String>) -> Self;
    pub fn enter_edit_mode(&mut self);
    pub fn render(&self, area: Rect, buf: &mut Buffer);
}
```
- Display proposed git command
- Show controls: `[Enter] Execute  [E] Edit  [Esc] Cancel`
- Edit mode: allow modifying command before execution
- Visual distinction between proposed and edited commands

**Output Display** (`src/ui/output.rs`):
- Scrollable text area for command output
- Show both stdout and stderr
- Color-code errors (red) and success (green)
- Display execution status (success/failure with exit code)

**Application Flow**:
1. User types query in input widget
2. Show "Translating..." indicator
3. Call LLM translator (async)
4. Display proposed command in preview widget
5. Wait for user action:
   - Enter: execute command
   - E: enter edit mode
   - Esc: cancel
6. Execute command via GitExecutor
7. Show output in output widget
8. Refresh repository state
9. Clear input and preview for next query

**Async Integration**:
```rust
// Use tokio for async LLM calls
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut app = App::new()?;
    app.run().await?;
    Ok(())
}
```

#### Acceptance Criteria
- [ ] User can type natural language query in input widget
- [ ] Pressing Enter submits query and shows loading indicator
- [ ] Proposed command appears in preview widget
- [ ] User can press Enter to execute command
- [ ] User can press 'E' to edit command before execution
- [ ] User can press Esc to cancel and return to input
- [ ] Executed command output appears in output widget
- [ ] Repository panel refreshes after command execution
- [ ] Works in offline mode (direct git command input)

#### Test Plan
- **Unit Tests**:
  - Input widget key handling
  - Command preview state transitions
  - Output formatting

- **Integration Tests**:
  - Full flow: input → translation → preview → execution → output
  - Test with both online (LLM) and offline modes

- **Manual Testing**:
  - Complete git workflow: stage files, commit, create branch
  - Test command editing
  - Test cancellation
  - Verify output display accuracy

#### Rollback Strategy
Revert to Phase 2 (display-only TUI) by removing input/command execution features.

#### Risks
- **Risk**: UI responsiveness during LLM API calls
  - **Mitigation**: Async execution with loading indicator; UI remains interactive
- **Risk**: Command editing UX is confusing
  - **Mitigation**: Clear visual feedback; test with users; consider in-line editing

---

### Phase 5: Error Handling & Safety

**Status**: pending
**Dependencies**: Phase 4 (command execution)

#### Objectives
- Implement command validation against allowlist
- Add dangerous operation detection and confirmation
- Translate git errors to plain language
- Implement audit logging
- Add security measures to prevent command injection

#### Deliverables
- [ ] `src/security/validator.rs` - Command validation logic
- [ ] `src/security/dangerous_ops.rs` - Dangerous operation detection
- [ ] `src/error/translator.rs` - Git error to plain language translation with raw error toggle
- [ ] `src/ui/error_display.rs` - Error display widget with 'T' toggle key support
- [ ] `src/audit/logger.rs` - Command audit logging
- [ ] Dangerous operation confirmation dialog
- [ ] Security tests (command injection attempts)

#### Implementation Details

**Command Validator** (`src/security/validator.rs`):
```rust
pub struct CommandValidator {
    allowed_subcommands: HashSet<&'static str>,
    dangerous_flags: HashSet<&'static str>,
}

impl CommandValidator {
    pub fn validate(&self, command: &str) -> Result<ValidatedCommand, ValidationError>;
    fn check_subcommand(&self, cmd: &str) -> bool;
    fn check_for_injection(&self, cmd: &str) -> Result<(), ValidationError>;
    fn detect_dangerous_ops(&self, cmd: &str) -> Option<DangerousOp>;
}

pub struct ValidatedCommand {
    pub command: String,
    pub is_dangerous: bool,
    pub danger_type: Option<DangerousOp>,
}

pub enum DangerousOp {
    ForcePush,
    HardReset,
    Clean,
    FilterBranch,
}
```
- Validate against allowlist from specification (status, log, commit, push, etc.)
- Check for suspicious operators: `;`, `||`, `|`, `>`, `<`, `$()`, backticks
- Detect `--exec` and `-c core.sshCommand` flags
- Allow `&&` only between git commands
- Regex validation: `^git\s+(subcommand)\s+[flags and args]$`

**Dangerous Operation Handler** (`src/security/dangerous_ops.rs`):
```rust
pub struct DangerousOpConfirmation {
    operation: DangerousOp,
    command: String,
    confirmed: bool,
}

impl DangerousOpConfirmation {
    pub fn render(&self, area: Rect, buf: &mut Buffer);
    pub fn handle_input(&mut self, input: &str) -> bool;
}
```
- Show warning dialog with operation details
- Require user to type "CONFIRM" to proceed
- Clear visual distinction (red border, warning icon)

**Error Translator** (`src/error/translator.rs`):
```rust
pub struct ErrorTranslator;

impl ErrorTranslator {
    pub fn translate(error: &GitError) -> UserFriendlyError;
}

pub struct UserFriendlyError {
    pub simple_message: String,
    pub suggestion: Option<String>,
    pub raw_error: String,
    pub show_raw: bool,  // Toggle state for displaying raw error
}
```
- Pattern matching on common git errors:
  - "no upstream branch" → "No remote branch is set up. Push with -u flag?"
  - "merge conflict" → "Merge has conflicts. View conflicts or abort merge?"
  - "detached HEAD" → "Not on any branch. Create a branch or checkout existing branch."
- Provide recovery suggestions
- Default to simplified error message, toggle with 'T' key to show/hide raw error
- Visual indicator when raw error is shown: "Press 'T' to hide technical details"

**Audit Logger** (`src/audit/logger.rs`):
```rust
pub struct AuditLogger {
    log_path: PathBuf,
}

impl AuditLogger {
    pub fn log_command(&self, command: &str, result: &CommandResult) -> Result<(), IoError>;
}
```
- Log to `~/.config/gitalky/history.log`
- Format: `[timestamp] [user] [repo_path] [command] [exit_code]`
- Rotate log file if exceeds 10MB

#### Acceptance Criteria
- [ ] Only allowlisted git subcommands are accepted
- [ ] Commands with suspicious operators are rejected with clear error
- [ ] Dangerous operations trigger confirmation dialog
- [ ] User must type "CONFIRM" for dangerous operations
- [ ] Git errors are translated to plain language with suggestions
- [ ] Raw error is accessible via 'T' toggle key
- [ ] Error display shows visual indicator for toggle state
- [ ] Default shows simplified error, 'T' key switches between simple/raw
- [ ] All executed commands are logged to history.log
- [ ] Command injection attempts are blocked

#### Test Plan
- **Unit Tests**:
  - Validator with allowed and disallowed commands
  - Injection detection with malicious inputs
  - Error translation with sample git errors
  - Dangerous operation detection

- **Integration Tests**:
  - Attempt command injection (should fail)
  - Execute dangerous operation (should require confirmation)
  - Trigger git error (should show friendly message)

- **Security Tests**:
  - Try to execute shell commands: `git status; rm -rf /`
  - Try to use pipes: `git log | sh`
  - Try to use redirection: `git status > /etc/passwd`
  - All should be blocked

- **Manual Testing**:
  - Test dangerous operations: force push, hard reset
  - Trigger common git errors and verify translations
  - Check audit log for command history

#### Rollback Strategy
Disable validation temporarily (log warnings instead of blocking), remove dangerous op confirmation.

#### Risks
- **Risk**: Validation is too strict, blocks legitimate commands
  - **Mitigation**: Comprehensive test suite with real-world git usage; allow user override with warning
- **Risk**: Error translation misses important details from raw error
  - **Mitigation**: Always provide access to raw error; improve translations based on user feedback

---

### Phase 6: Configuration & First-Run Experience

**Status**: pending
**Dependencies**: Phase 3 (LLM), Phase 5 (offline mode detection)

#### Objectives
- Implement configuration file parsing
- Create first-run setup wizard
- Implement offline mode detection and switching
- Add configuration for LLM provider, model, and preferences
- Polish user experience

#### Deliverables
- [ ] `src/config/settings.rs` - Configuration file handling
- [ ] `src/config/first_run.rs` - First-run setup wizard
- [ ] `src/ui/setup_wizard.rs` - Setup UI screens
- [ ] Configuration file with validation
- [ ] Offline mode detection and mode switching
- [ ] Help screen with keyboard shortcuts
- [ ] README.md with setup instructions

#### Implementation Details

**Configuration** (`src/config/settings.rs`):
```rust
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub llm: LLMConfig,
    pub ui: UIConfig,
    pub behavior: BehaviorConfig,
    pub git: GitConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LLMConfig {
    pub provider: String,
    pub model: String,
    pub api_key_env: String,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError>;
    pub fn save(&self) -> Result<(), ConfigError>;
    pub fn default_config() -> Self;
}
```
- Load from `~/.config/gitalky/config.toml`
- Validate configuration values
- Create with defaults if missing
- Use `toml` crate for parsing

**First-Run Setup** (`src/config/first_run.rs`):
```rust
pub struct FirstRunWizard {
    step: SetupStep,
}

pub enum SetupStep {
    Welcome,
    SelectProvider,
    EnterAPIKey,
    ValidateAPI,
    Complete,
}

impl FirstRunWizard {
    pub fn run() -> Result<Config, SetupError>;
}
```
- Welcome screen with ASCII art (from spec wireframe)
- Provider selection: [1] Anthropic [2] OpenAI (coming soon) [3] Local/Ollama (coming soon) [4] Skip
- API key input or environment variable selection
- API validation (test request with 5s timeout)
- Retry/skip on validation failure
- Create config file with 600 permissions
- Create history.log

**Offline Mode** (`src/ui/app.rs` enhancement):
```rust
pub enum AppMode {
    Online,
    Offline,
}

impl App {
    pub fn detect_mode(&self) -> AppMode;
    pub fn switch_to_offline(&mut self);
    pub async fn try_reconnect(&mut self) -> Result<(), ConnectionError>;
}
```
- Check API connectivity on startup (2s timeout)
- Show [ONLINE] or [OFFLINE] indicator in status bar
- Change input prompt based on mode
- 'R' key to retry connection
- Graceful degradation: disable natural language input in offline mode

**Help Screen** (`src/ui/help.rs`):
```
╔════════════════════════════════════════════════════════════════╗
║                      Gitalky Help                              ║
╠════════════════════════════════════════════════════════════════╣
║ Keyboard Shortcuts:                                            ║
║   q          Quit application                                  ║
║   ?          Show this help                                    ║
║   Esc        Cancel current operation                          ║
║   Enter      Submit query / Execute command                    ║
║   E          Edit proposed command                             ║
║   T          Toggle raw/simplified error display               ║
║   R          Retry LLM connection (when offline)               ║
║                                                                 ║
║ Example Queries:                                               ║
║   "show me what changed"                                       ║
║   "commit these changes with message 'fix bug'"                ║
║   "create a new branch called feature-x from main"             ║
║   "show me the last 10 commits"                                ║
║                                                                 ║
║ Configuration: ~/.config/gitalky/config.toml                   ║
║ Audit Log: ~/.config/gitalky/history.log                       ║
╚════════════════════════════════════════════════════════════════╝
```

**Documentation** (`README.md`):
- Installation instructions
- First-run setup guide
- Configuration reference
- Example usage scenarios
- Troubleshooting section
- Security considerations

#### Acceptance Criteria
- [ ] First-run wizard appears when no config file exists
- [ ] User can select LLM provider and enter API key
- [ ] API key validation succeeds with valid key
- [ ] Config file is created with 600 permissions
- [ ] App starts in offline mode if no API key configured
- [ ] Offline mode indicator shows in status bar
- [ ] User can press 'R' to retry connection
- [ ] Help screen appears when pressing '?'
- [ ] Configuration file is properly validated on load
- [ ] README.md includes complete setup instructions

#### Test Plan
- **Unit Tests**:
  - Config file parsing and validation
  - Default config generation
  - Permission setting on config file

- **Integration Tests**:
  - First-run wizard flow (simulated user input)
  - Config load/save round-trip
  - Offline mode detection and switching

- **Manual Testing**:
  - Delete config file and run app (first-run experience)
  - Test with invalid API key (should show error, allow retry)
  - Test offline mode functionality
  - Verify help screen content
  - Test config file editing and reload

#### Rollback Strategy
Use hardcoded defaults, skip first-run setup for development.

#### Risks
- **Risk**: First-run setup is confusing for users
  - **Mitigation**: Clear prompts, examples, and help text; user testing
- **Risk**: Config file corruption breaks app startup
  - **Mitigation**: Validate config on load; fall back to defaults with warning

---

## Dependency Map
```
Phase 1 (Foundation & Git Integration)
    ↓
Phase 2 (TUI Framework)    Phase 3 (LLM Integration)
    ↓                              ↓
Phase 4 (Command Confirmation & Execution)
    ↓
Phase 5 (Error Handling & Safety)
    ↓
Phase 6 (Configuration & First-Run)
```

## Resource Requirements

### Development Resources
- **Engineers**: Single Rust developer with knowledge of:
  - Async Rust (tokio)
  - TUI development (ratatui/crossterm)
  - Git internals
  - LLM API integration
- **Environment**:
  - Development machine with Git 2.20+
  - Anthropic API key for testing
  - Various test git repositories

### Infrastructure
- No infrastructure requirements (local application)
- Test git repositories for automated testing
- CI/CD for running tests (future)

## Integration Points

### External Systems
- **System**: Anthropic Claude API
  - **Integration Type**: REST API (HTTPS)
  - **Phase**: Phase 3
  - **Fallback**: Offline mode (direct git command input)

- **System**: Git CLI
  - **Integration Type**: Shell command execution
  - **Phase**: Phase 1
  - **Fallback**: Error message if git not installed

### Internal Systems
- All components are internal to the gitalky application
- No external database or message queues

## Risk Analysis

### Technical Risks
| Risk | Probability | Impact | Mitigation | Owner |
|------|------------|--------|------------|-------|
| LLM returns dangerous commands | Medium | High | Command validation, user review, allowlist | Phase 5 |
| Git output parsing breaks | Medium | Medium | Porcelain formats, comprehensive tests, graceful errors | Phase 1 |
| UI performance with large repos | Low | Medium | Pagination, lazy loading, async refresh | Phase 2 |
| Command injection vulnerability | Low | Critical | Strict validation, no shell interpolation, security tests | Phase 5 |
| API rate limits during testing | Medium | Low | Request caching, test mode, usage monitoring | Phase 3 |
| Terminal compatibility issues | Low | Medium | Test multiple terminals, graceful degradation | Phase 2 |

### Implementation Risks
| Risk | Probability | Impact | Mitigation | Owner |
|------|------------|--------|------------|-------|
| Underestimated complexity of async TUI | Medium | Medium | Start simple, iterate; seek help if needed | Phase 4 |
| Error translation misses edge cases | High | Low | Iterative improvement, user feedback, raw error access | Phase 5 |
| First-run UX is confusing | Medium | Medium | User testing, clear prompts, help text | Phase 6 |

## Validation Checkpoints

1. **After Phase 1**:
   - Verify git command execution and parsing work with real repositories
   - Test with various repository states

2. **After Phase 2**:
   - Validate UI displays repository state accurately
   - Verify performance with large repository

3. **After Phase 3**:
   - Test LLM translation accuracy with 20+ common operations
   - Verify context building stays within token budget

4. **After Phase 4**:
   - Complete full workflow: query → translation → execution → result
   - Verify repository state refreshes correctly

5. **After Phase 5**:
   - Security review: test command injection attempts
   - Verify dangerous operations require confirmation

6. **Before Phase 6 Complete**:
   - End-to-end testing of complete application
   - User acceptance testing with real developers

## Monitoring and Observability

### Metrics to Track (Future Enhancement)
- LLM API request count and latency
- Command translation accuracy (user edits as proxy)
- Most common natural language queries
- Error rate by git operation type

### Logging Requirements
- **Audit Log** (`~/.config/gitalky/history.log`):
  - All executed git commands
  - Timestamp, repository path, command, exit code
  - Rotate when exceeds 10MB

- **Application Log** (stderr during development):
  - Errors and warnings
  - LLM API failures
  - Git command failures

### Alerting
Not applicable for local CLI application. Errors shown directly to user in TUI.

## Documentation Updates Required
- [ ] README.md with installation and setup
- [ ] Architecture diagram (component structure)
- [ ] Configuration reference (config.toml options)
- [ ] Security considerations document
- [ ] Contributing guide (future)
- [ ] Inline code documentation (rustdoc)

## Post-Implementation Tasks
- [ ] Performance validation with large repositories (>10K files)
- [ ] Security audit (manual command injection testing)
- [ ] User acceptance testing with 5+ developers
- [ ] Collect feedback on translation accuracy
- [ ] Measure actual token usage and API costs
- [ ] Test with different terminal emulators

## Self-Review Notes

**Planning Approach**:
- Broke implementation into 6 phases based on logical dependencies
- Each phase delivers independently testable value
- Phases build progressively: foundation → display → intelligence → interaction → safety → polish
- Avoided overly granular phases that would slow progress
- Security and error handling in dedicated phase to ensure comprehensive coverage

**Key Decisions**:
1. **Phase ordering**: Git integration first (foundation), then TUI (visualization), then LLM (intelligence)
2. **Async integration**: Deferred until Phase 4 when actually needed for LLM calls
3. **Security**: Dedicated phase to ensure comprehensive validation and testing
4. **Configuration**: Last phase as it depends on understanding offline mode needs

**Potential Concerns**:
- Phase 4 combines command confirmation and execution - could be split if too complex
- Query classification in Phase 3 uses simple heuristics - may need refinement
- First-run wizard in Phase 6 could be complex - might need extra attention

**Test Coverage Strategy**:
- Unit tests for all logic components (parsers, validators, context builders)
- Integration tests for git operations and LLM calls
- Security tests for command injection attempts
- Manual testing for UX flows and visual validation

**Confidence Level**: High (90%) - Plan is comprehensive and follows logical progression

## Approval
- [ ] Technical Lead Review
- [ ] Human Review and Approval to Proceed
- [ ] Resource Allocation Confirmed

## Change Log
| Date | Change | Reason | Author |
|------|--------|--------|--------|
| 2025-10-04 | Initial plan creation | Based on approved specification 0002 | Claude |
| 2025-10-04 | Fixed edition to 2024, added async-trait | Critical: align with spec requirements | Claude |
| 2025-10-04 | Added git version detection | Important: validate git 2.20+ requirement | Claude |
| 2025-10-04 | Added stash support to Phase 2 | Important: missing from spec requirements | Claude |
| 2025-10-04 | Added raw error toggle UI | Important: spec requires raw error access | Claude |
| 2025-10-04 | Added token budget enforcement | Important: prevent context overflow | Claude |
| 2025-10-04 | Added minor issue recommendations | Document optional enhancements | Claude |
| 2025-10-05 | Added file path context to Phase 3 | Enhancement: enable fuzzy file matching | Claude |
| 2025-10-05 | Added file path matching to LLM prompt | Enhancement: improve command accuracy | Claude |
| 2025-10-05 | Added V2 iterative clarification note | Documentation: defer clarification to V2 | Claude |

## Recommendations for Minor Issues

These issues are **not blockers** for v1 but should be considered for future iterations:

### 7. Performance Benchmarks
**Issue**: No explicit performance benchmark tests despite requirements (<100ms refresh, <500ms startup)

**Recommendation**:
- Add to **Phase 2** post-implementation tasks:
  - Create benchmark test: `benches/repo_state_refresh.rs`
  - Use `criterion` crate for benchmarking
  - Test with repos of various sizes (100, 1K, 10K files)
  - Set CI performance regression alerts
- **Alternative**: Manual timing checks during Phase 2 evaluation, add benchmarks in v1.1

### 8. API Key Storage Preference
**Issue**: Config file storage conflicts with spec preference for environment variables

**Recommendation**:
- **Phase 6** first-run wizard should:
  - **Default**: Prompt for environment variable name (don't store key in file)
  - **Option**: "Or press 'S' to store in config file (less secure)"
  - Store only `api_key_env` in config by default
  - Fall back to `api_key` field in config only if user explicitly chooses
- This aligns with security best practices

### 9. Concurrent Repository Changes Detection
**Issue**: No mechanism to detect stale repository state during long LLM operations

**Recommendation**:
- **Phase 4** enhancement (optional for v1):
  - Add repository state hash/timestamp
  - Before executing command, check if state changed
  - Show warning: "Repository changed during translation. Refresh and retry? [Y/n]"
- **Alternative**: Document as known limitation, address in v1.1 if users report issues

### 10. Large Output Handling
**Issue**: No pagination/truncation for huge git outputs (e.g., `git log` on 10K+ commit repo)

**Recommendation**:
- **Phase 4** output display:
  - Implement scrollable buffer with max size (10K lines)
  - Truncate with indicator: "... (5000 more lines, use git command directly for full output)"
  - For memory safety, limit stdout/stderr capture to 10MB
- Add to Phase 4 acceptance criteria

### 11. API Mocking Strategy
**Issue**: Testing without API key needs mock/stub strategy for Phase 3

**Recommendation**:
- **Phase 3** testing approach:
  - Use trait-based design (already planned with `LLMClient` trait)
  - Create `MockLLMClient` for tests
  - Use feature flag: `#[cfg(test)]` for mock client
  - Real API integration tests gated by env var: `GITALKY_API_KEY`
- Document in Phase 3 test plan

### 12. Panic Recovery Detail
**Issue**: Terminal state restoration on panic mentioned but not detailed

**Recommendation**:
- **Phase 2** terminal management:
  - Use `scopeguard` or panic hook to ensure cleanup
  - Pattern:
    ```rust
    let mut terminal = setup_terminal()?;
    let cleanup = scopeguard::guard((), |_| {
        restore_terminal(&mut terminal).ok();
    });
    // ... run app ...
    drop(cleanup);
    ```
  - Add panic test to verify terminal restoration

## Notes
- This is a v1 implementation - focus on core functionality over polish
- Multi-step workflows deferred to v2 (as specified)
- Windows support deferred to v2 (Unix-like systems only for v1)
- Additional LLM providers (OpenAI, Ollama) are future enhancements
- Performance optimization can be done iteratively if needed
- Consider user feedback for v2 feature prioritization
- Minor issues (#7-12 above) can be addressed during implementation or deferred to v1.1

### V2 Feature: Iterative Clarification Flow

**Scope**: Not included in V1 implementation, documented for future reference

**Purpose**: Allow users to provide additional context when LLM-generated commands fail, enabling conversational refinement

**Key Components** (see spec section "Iterative Clarification Flow (V2)"):
1. New `AppState::Clarifying` state for entering clarification text
2. Enhanced `GitCommand` struct to carry error context
3. New `build_clarification_context()` method in ContextBuilder
4. UI changes to show 'c' key option after command failures
5. Clarification input widget (can reuse existing InputWidget with different prompt)

**Implementation Notes**:
- V1 includes enhanced file path context (full paths in default context)
- V1 includes file path fuzzy matching instructions in LLM prompt
- These improvements reduce the need for clarification in many cases
- V2 will add iterative clarification as a fallback when initial translation fails

**Decision Rationale**:
- V1 focuses on getting the command right the first time via better context
- V2 adds conversational refinement for edge cases
- This phasing allows us to validate the effectiveness of enhanced context before adding clarification complexity
