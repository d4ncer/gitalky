# Gitalky Architecture Summary

## Overview

Gitalky is a **Terminal UI (TUI) application** that translates natural language into git commands using LLMs, with a focus on transparency, safety, and learning. The architecture follows a **layered, modular design** with clear separation of concerns.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     main.rs                              │
│  (Entry point, terminal setup, panic handling)           │
└─────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────┐
│                   ui::App                                │
│  (Main application state machine & event loop)           │
└─────────────────────────────────────────────────────────┘
         │          │          │          │
         ▼          ▼          ▼          ▼
    ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐
    │  Git   │ │  LLM   │ │Security│ │ Config │
    │ Layer  │ │ Layer  │ │ Layer  │ │ Layer  │
    └────────┘ └────────┘ └────────┘ └────────┘
```

## Core Design Decisions

### 1. **Why main.rs is Thin** ✅

**Decision**: Keep `main.rs` minimal - only initialization and terminal management.

**Rationale**:
- **Testability**: Complex logic in `main()` is hard to test (requires process-level testing)
- **Separation of Concerns**: Entry point should only handle:
  - Git version validation
  - Configuration loading
  - Repository discovery
  - Terminal setup/cleanup
  - Panic handling (restore terminal on crash)
- **Error Handling**: Early validation fails fast with clear error messages to stderr
- **Clean Exit**: Ensures terminal is always restored, even on panic

**What main.rs Does**:
1. Validates git version (≥2.20 required)
2. Loads config or runs first-run wizard
3. Discovers git repository
4. Sets up panic hook to restore terminal
5. Initializes terminal (raw mode, alternate screen)
6. Creates `App` and runs event loop
7. Restores terminal on exit

**What main.rs Does NOT Do**:
- ❌ Business logic
- ❌ Event handling
- ❌ State management
- ❌ LLM communication
- ❌ Git command execution

### 2. **Layered Module Architecture** ✅

**Decision**: Organize code into clear functional layers with minimal coupling.

```
src/
├── main.rs              # Entry point only
├── lib.rs               # Public API exports
├── error.rs             # Unified error types
├── git/                 # Git operations layer
│   ├── repository.rs    # Repository state & discovery
│   ├── executor.rs      # Command execution & sanitization
│   ├── parser.rs        # Git output parsing
│   └── version.rs       # Version detection & validation
├── llm/                 # LLM integration layer
│   ├── client.rs        # Trait for LLM clients
│   ├── anthropic.rs     # Claude API implementation
│   ├── translator.rs    # NL → Git command translation
│   └── context.rs       # Repository context for LLM
├── security/            # Security & validation layer
│   └── validator.rs     # Command validation & dangerous op detection
├── config/              # Configuration layer
│   ├── settings.rs      # Config file management
│   └── first_run.rs     # Interactive setup wizard
├── error_translation/   # User-friendly error messages
│   └── translator.rs    # Git errors → Plain language
├── audit/               # Audit logging layer
│   └── logger.rs        # Command execution logging
└── ui/                  # Terminal UI layer
    ├── app.rs           # Main application state machine
    ├── input.rs         # Input widget
    ├── output.rs        # Output display
    ├── repo_panel.rs    # Repository state panel
    ├── command_preview.rs # Command preview widget
    └── help.rs          # Help screen
```

**Rationale**:
- **Maintainability**: Each module has a single responsibility
- **Testability**: Layers can be tested independently
- **Extensibility**: Easy to add new LLM providers or UI widgets
- **Reusability**: Core logic (git, llm) is usable outside TUI

### 3. **Error Handling Architecture** ✅

**Decision**: Unified error type (`AppError`) that wraps all module-specific errors.

```rust
pub enum AppError {
    Git(GitError),              // Git operations
    Config(ConfigError),         // Configuration
    Llm(LLMError),              // LLM communication
    Translation(TranslationError), // Query translation
    Security(ValidationError),   // Validation failures
    Setup(SetupError),          // First-run setup
    Io(io::Error),              // I/O operations
}
```

**Rationale**:
- **Type Safety**: Compiler enforces error handling at boundaries
- **Automatic Conversion**: `From` trait allows `?` operator to work seamlessly
- **Error Context**: Preserves full error chain via `source()`
- **User-Friendly**: Separate translation layer for plain-language errors
- **Centralized**: Single place to handle all error types

**Flow**:
```
GitError::CommandFailed
  → AppError::Git(...)        (automatic via From trait)
  → ErrorTranslator::translate_app_error()
  → UserFriendlyError { simple_message, suggestion, raw_error }
```

### 4. **State Machine Pattern in App** ✅

**Decision**: UI state is managed as an explicit state machine.

```rust
enum AppState {
    Input,              // User typing query
    Translating,        // Waiting for LLM response
    Preview,            // Showing proposed command
    ConfirmDangerous,   // Confirming dangerous operation
    Executing,          // Running command
    ShowingOutput,      // Displaying command output
}
```

**Rationale**:
- **Clarity**: State transitions are explicit and documented
- **Safety**: Prevents invalid state combinations (e.g., can't execute while translating)
- **Debugging**: Current state is always visible
- **Event Handling**: Different states handle different key events
- **Async Coordination**: State tracks whether waiting for LLM/git

**State Flow**:
```
Input → Translating → Preview → [ConfirmDangerous?] → Executing → ShowingOutput → Input
   ↑_______________________________________________________________|
```

### 5. **Offline-First Design** ✅

**Decision**: Full functionality without LLM, graceful degradation when offline.

**Modes**:
```rust
enum AppMode {
    Normal,   // LLM available, natural language enabled
    Offline,  // No LLM, direct git command input only
}
```

**Rationale**:
- **Resilience**: App works even if LLM API is down
- **Privacy**: Some users may prefer not to send data to LLM
- **Speed**: Direct commands bypass LLM latency
- **Learning**: Offline mode encourages learning git syntax

**Offline Capabilities**:
- ✅ Repository state display (all read operations)
- ✅ Direct git command execution
- ✅ Command history and output
- ✅ All UI features except NL translation

**Online-Only**:
- Natural language → git command translation
- Context-aware suggestions
- Error simplification (uses raw git errors in offline mode)

### 6. **Security-First Command Execution** ✅

**Decision**: Multi-layer validation before executing any git command.

**Validation Layers**:
1. **Sanitization** (Executor): Remove shell metacharacters (`;`, `|`, `&`, `$()`, `` ` ``)
2. **Environment Sanitization** (Executor): Remove dangerous git env vars via `env_clear()`, then re-add safe ones (PATH, HOME, USER, LOGNAME, LANG, LC_ALL, TZ, TERM, TMPDIR)
3. **Validation** (Validator): Check against hardcoded allowlist of git subcommands (not user-configurable)
4. **Dangerous Operation Detection** (Validator): Identify destructive commands
5. **User Confirmation**: Require explicit confirmation for dangerous ops
6. **Audit Logging**: Log all executed commands (if enabled)

```rust
// Validation flow
user_input (or LLM output)
  → CommandValidator::validate(command)
  → ValidatedCommand { command, is_dangerous, danger_type }
  → if dangerous: require_confirmation()
  → GitExecutor::execute(validated_command)
      ↳ Defense-in-depth: sanitize input, clean env vars, parse to Vec<String>
  → AuditLogger::log(command, result)
```

**Dangerous Operations** (require confirmation):
- `push --force`, `push -f` ✅
- `reset --hard` ✅
- `clean -fd`, `clean -fdx` ✅
- `rebase` (interactive or not) ✅
- `branch -D`, `branch -d` ✅
- `checkout --force`, `checkout -f` ✅
- `filter-branch` ✅

**Security Strengths**:
- ✅ **Defense-in-Depth**: Both Validator and Executor sanitize inputs
- ✅ **Allowlist Approach**: Only permitted git subcommands allowed
- ✅ **Environment Hardening**: Dangerous env vars stripped before execution
- ✅ **Comprehensive Dangerous Op Detection**: All major destructive operations covered

**Security Gaps**:
- ✅ ~~**No LLM Output Validation**~~ **FIXED (2025-10-09)**: 7 validation checks prevent gibberish/injection
- ⚠️ **Command Injection via Flags**: Doesn't validate all flag combinations (e.g., `--upload-pack`, `-C`)

**Rationale**:
- **Safety**: Prevents accidental data loss
- **Transparency**: User always sees what will execute
- **Education**: Warnings teach about destructive operations
- **Compliance**: Audit log for enterprise use cases

### 7. **Context-Aware LLM Integration** ⚠️

**Decision**: Build rich repository context before LLM calls, with token budget enforcement.

**Context Builder Strategy**:
```rust
pub enum QueryType {
    Commit,   // Needs staged/unstaged file details
    Branch,   // Needs upstream tracking info
    Diff,     // Needs file change details
    History,  // Needs recent commits
    Stash,    // Needs stash list
    General,  // Minimal context
}
```

**Flow**:
1. Classify user query → QueryType
2. Build default context (~500 tokens): branch, files, commits count
3. Build escalated context for QueryType (~2000 tokens): detailed info
4. ~~Enforce 5000 token budget (truncate if needed)~~ **⚠️ NOT IMPLEMENTED**
5. Send context + query to LLM
6. Receive git command **⚠️ WITHOUT SANITY CHECKING** (basic validation happens in Validator layer)

**Rationale**:
- **Accuracy**: LLM has repo state, makes better suggestions
- **Token Efficiency**: Attempt to send only relevant context (escalation strategy per QueryType)
- **Cost Control**: ~~Hard limit on token usage~~ **⚠️ NO ACTUAL BUDGET ENFORCEMENT**
- **Performance**: Minimal context = faster responses

**Known Limitations**:
- ⚠️ **No token budget enforcement**: Code estimates tokens but doesn't enforce limits
- ✅ **LLM output validation**: IMPLEMENTED (2025-10-09) - 7 validation checks in translator.rs
- ✅ **Hallucination detection**: IMPLEMENTED (2025-10-09) - Rejects explanation text and non-git commands

**Example Escalation**:
- Query: "commit my changes" → QueryType::Commit
- Escalated context adds: Staged files with paths, unstaged files with status
- LLM sees: "staged: src/main.rs (Modified), unstaged: README.md (Modified)"
- Result: More accurate commit command

### 8. **Async Architecture** ⚠️

**Decision**: Use Tokio async runtime for LLM calls, keep UI responsive.

```rust
#[tokio::main]
async fn main() -> io::Result<()> {
    // ...
    let mut app = App::new(repo, config)?;
    app.run(&mut terminal).await
}
```

**Rationale**:
- **Responsiveness**: UI doesn't block during LLM API calls (can take 1-3 seconds)
- ~~**Concurrency**: Could add features like background state refresh~~ **⚠️ NOT UTILIZED**
- **Network I/O**: HTTP requests are naturally async
- **Future-Proof**: Easy to add more async operations

**Current Async Operations**:
- LLM translation (anthropic API calls)
- First-run wizard (async for consistency)

**Synchronous Operations**:
- Git command execution (via `std::process::Command`) **⚠️ BLOCKS UI THREAD**
- UI rendering (immediate)
- File I/O (config, audit logs) **⚠️ COULD BLOCK ON SLOW/FULL FILESYSTEM**
- Repository state refresh (via `git status`) **⚠️ CAN TAKE 100ms-1s ON LARGE REPOS**

**Known Limitations**:
- ⚠️ **Async theater**: Most operations are synchronous; Tokio is only used for HTTP
- ⚠️ **Git blocks UI**: Long-running git commands freeze the UI
- ✅ **State refresh debounced**: Now uses 1-second debouncing + dirty detection (fixed)

### 9. **Widget Composition Pattern** ✅

**Decision**: UI is composed of reusable, testable widgets.

**Widget Hierarchy**:
```
App
├── InputWidget          (query input, mode indicator)
├── RepositoryPanel      (branch, status, files, commits)
├── CommandPreview       (proposed command, explanation)
├── OutputDisplay        (command results, scroll)
└── HelpScreen           (keyboard shortcuts, features)
```

**Rationale**:
- **Reusability**: Widgets can be used in different layouts
- **Testability**: Each widget has unit tests for logic
- **Encapsulation**: Widget state is self-contained
- **Layout Flexibility**: Easy to reorganize UI

**Widget Responsibilities**:
- **InputWidget**: Input capture, cursor management, mode display
- **RepositoryPanel**: Format git state for display (Magit-inspired)
- **CommandPreview**: Show command with syntax highlighting
- **OutputDisplay**: Scroll, format output (success/error coloring)
- **HelpScreen**: Contextual help based on mode/state

### 10. **Configuration Management** ✅

**Decision**: TOML config with first-run wizard for setup.

**Config Structure**:
```toml
[llm]
provider = "anthropic"
api_key = "sk-..."
model = "claude-sonnet-4-20250514"

[behavior]
confirm_dangerous = true
log_commands = true
show_raw_errors = false
```

**First-Run Experience**:
1. Detect no config → run wizard
2. Prompt for LLM provider
3. Prompt for API key (validation requires network access to test API)
4. Save to `~/.config/gitalky/config.toml`
5. Start app

**Rationale**:
- **User-Friendly**: No manual config file editing
- **Validation**: API key tested via actual API call before saving
- **Flexibility**: Users can edit TOML manually later
- **Portability**: XDG-compliant config location

## Architecture Trade-offs

### ✅ Chosen Trade-offs

| Trade-off | Choice | Why |
|-----------|--------|-----|
| **Library vs Framework** | Library-based (ratatui) | More control, better testing |
| **Sync vs Async** | Hybrid (async for LLM, sync for git) | Balance between responsiveness and simplicity |
| **Direct Git vs libgit2** | Direct git commands | Full compatibility, easier parsing |
| **Centralized vs Distributed State** | Centralized in App | Simpler state management for TUI |
| **Eager vs Lazy Loading** | Eager (load state on startup) | Better UX, repo state needed immediately |

### ⚠️ Known Limitations

#### Design & Testing
1. **TUI Testing**: UI code is hard to unit test (requires terminal), covered by manual QA
2. **Git Output Parsing**: Relies on git porcelain format stability
3. **Platform Support**: Unix-like systems initially (Windows support future work)

#### Performance
4. **Git Blocks UI Thread**: Long-running git commands (e.g., `git log` on large repos) freeze UI
5. ✅ **State Refresh Optimized**: Now uses 1-second debouncing + dirty detection (previously refreshed every 100ms)
6. **Quote Parser Limitations**: Executor's custom quote parser doesn't support escape sequences (`\"`, `\'`) - this is acceptable because git commands rarely need them and security is prioritized

#### LLM Integration
6. **No Token Budget Enforcement**: Code estimates tokens but doesn't actually enforce limits
7. ✅ **LLM Output Validation**: IMPLEMENTED (2025-10-09) - 7 validation checks (length, newlines, shell metacharacters, git command validation, explanation detection)
8. **LLM Latency**: 1-3 second delay for translation (shows "Translating..." state)
9. **Token Costs**: LLM usage incurs API costs

#### Security
10. ✅ **Command Validation**: Multi-layer validation with allowlist + dangerous operation detection
11. ✅ **Environment Sanitization**: Removes dangerous git environment variables
12. ✅ **LLM Hallucination Detection**: IMPLEMENTED (2025-10-09) - Rejects explanation text, shell metacharacters, non-git commands

## Design Patterns Used

### Creational
- **Builder Pattern**: ContextBuilder for LLM context
- **Factory Pattern**: LLM client creation based on provider

### Structural
- **Facade Pattern**: App wraps complex subsystems
- **Adapter Pattern**: ErrorTranslator adapts errors for UI

### Behavioral
- **State Pattern**: AppState enum for UI flow
- **Strategy Pattern**: Different context strategies per QueryType
- **Observer Pattern**: UI updates when repo state changes

### Rust-Specific
- **Newtype Pattern**: ValidatedCommand wraps String
- **Result/Option Chaining**: Extensive use of `?` operator
- **Trait Objects**: `Box<dyn LLMClient>` for polymorphism

## Recent Improvements (Post-Spec 0003)

Following critical review of the architecture, several security and performance issues were addressed:

### Security Hardening (2025-10-09)
- ✅ **LLM Output Validation**: Added 7 validation checks to prevent hallucinations and injection attacks
- ✅ **Environment Variable Sanitization**: Executor now uses `env_clear()` and only re-adds safe vars
- ✅ **Command Parsing Security**: Uses `Vec<String>` args instead of shell execution
- ✅ **Extended Dangerous Op Detection**: Added `branch -D`, `checkout --force`, `rebase` detection
- ✅ **Defense-in-Depth**: 3-layer security (LLM validation → Validator → Executor)

### Performance Optimization (2025-10-09)
- ✅ **State Refresh Debouncing**: Changed from 100ms polling to 1-second debouncing + dirty detection
- ✅ **Reduced CPU Usage**: 90% reduction in idle `git status` calls (600/min → 60/min)

### Test Coverage
- Tests increased from 182 to 217 (139 unit + 78 integration)
- Added 13 new LLM validation tests
- Added 13 new security integration tests
- Added 6 new dangerous operation tests
- Added 3 new executor sanitization tests

## Future Architectural Improvements

### Critical (Should Do Soon)
- ✅ ~~**LLM Output Validation**~~ **COMPLETED (2025-10-09)**: 7 validation checks in translator.rs
- [ ] **Token Budget Enforcement**: Actually enforce the 5000-token limit
- [ ] **Async Git Execution**: Move git commands off UI thread to prevent freezing
- [ ] **Shared Allowlist Constant**: Extract git subcommand allowlist to prevent duplication between validator and LLM validation

### Phase 4 (Planned)
- [ ] **Incremental State Updates**: Only refresh changed parts of repo state
- [ ] **Command History**: Store past translations for learning
- [ ] **Multi-Repo Support**: Switch between repositories in UI

### Phase 5 (Nice-to-Have)
- [ ] **Plugin System**: Allow custom LLM providers
- [ ] **Themes**: Customizable color schemes
- [ ] **Macro System**: Record and replay command sequences

## Key Architecture Principles

1. **Transparency**: Always show what will execute before running it ✅
2. **Safety**: Multi-layer validation, explicit confirmation for dangerous ops ✅
3. **Modularity**: Clear boundaries between git/llm/ui/security layers ✅
4. **Testability**: Core logic separated from UI for unit testing ✅
5. **Graceful Degradation**: Offline mode provides full git functionality ✅
6. **Error Clarity**: User-friendly messages with option to see technical details ✅
7. **Performance**: ⚠️ Sub-second git operations (not always true on large repos), token-efficient LLM calls (estimates only, no enforcement)

## References

- **Spec**: `codev/specs/0002-natural-language-git-tui.md`
- **Plan**: `codev/plans/0002-natural-language-git-tui.md`
- **Error Handling**: `docs/error_handling.md`
- **Testing Strategy**: `docs/testing_strategy.md`
- **Benchmarking**: `docs/benchmarking.md`

## Summary

Gitalky's architecture prioritizes **transparency, safety, and user experience** through:

- **Thin entry point** (main.rs) for testability
- **Layered modules** for maintainability
- **Unified error handling** with automatic conversions
- **State machine UI** for clear event flow
- **Offline-first design** for resilience
- **Security-first validation** for safety
- **Context-aware LLM** for accuracy
- **Async runtime** for responsiveness
- **Composable widgets** for reusability

This architecture evolved through **3 major phases** (Specs 0001-0003), resulting in a robust foundation with **217 tests** (139 unit + 78 integration), comprehensive **documentation**, and **performance benchmarking** infrastructure.
