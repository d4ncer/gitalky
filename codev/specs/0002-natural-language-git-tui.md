# Specification: Natural Language Git Terminal UI

## Metadata
- **ID**: 0002-natural-language-git-tui
- **Status**: approved
- **Created**: 2025-09-30
- **Last Updated**: 2025-10-04

## Clarifying Questions Asked
1. **Interaction Model**: Interactive UI with command input area and repo state panel
2. **Command Execution**: Show git command first, allow user to confirm/tweak before execution
3. **LLM Integration**: Configurable by user, default to Claude Sonnet 4.5
4. **Git Operations**: All operations (basic workflow, branching, history viewing)
5. **Repository Context**: Auto-detect from current directory, show branch/status in UI
6. **Error Handling**: Translate errors to simpler terms, show raw git errors on demand
7. **TUI Framework**: No preference (implementer's choice)
8. **Visual Design**: Model repo state panel on Magit's interface
9. **Safety**: Dangerous operations require special confirmation
10. **Configuration**: User-configurable LLM provider/model and preferences

## Problem Statement
Git has a steep learning curve with complex command syntax that requires memorization of numerous flags, options, and subcommands. Users often know what they want to accomplish ("make a new branch from main") but struggle to translate that intent into the correct git command syntax. Existing Git GUIs either oversimplify operations or hide what's happening, making them poor learning tools.

Terminal users need a tool that:
- Accepts natural language descriptions of git operations
- Translates intent into proper git commands
- Shows what will be executed before running it
- Provides visual feedback on repository state
- Maintains the transparency and power of the command line

## Current State
- Users must memorize git command syntax or constantly reference documentation
- Git CLI provides no visual context about repository state
- GUI tools hide command execution, limiting learning and customization
- No existing tool bridges natural language with git command execution in a terminal context

## Desired State
A terminal UI application (gitalky) that:
- Provides a Magit-inspired visual display of repository state
- Accepts natural language queries for git operations
- Translates queries to git commands using LLM
- Shows proposed commands for user review/modification
- Executes approved commands and updates the display
- Helps users learn git by showing the translation from intent to command
- Remains fully transparent about what operations are performed

## Stakeholders
- **Primary Users**: Terminal-based developers who use git regularly
- **Secondary Users**: Git learners who want to understand command translation
- **Technical Team**: Rust developers implementing the TUI
- **Business Owners**: Project maintainers

## Success Criteria
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
- [ ] **(V2)** Iterative clarification flow allows users to provide context when commands fail

## Constraints
### Technical Constraints
- Rust programming language (edition 2024)
- Must work in standard terminal emulators
- Must handle git repositories with various states (clean, dirty, conflicts, detached HEAD, etc.)
- LLM API calls introduce latency - UI must remain responsive
- Must parse git command output reliably

### Business Constraints
- LLM API usage incurs costs - should be mindful of token usage
- **Must work offline for local operations** (see "Offline Mode" section below)
- First version should be fully functional end-to-end

## Offline Mode

**Graceful Degradation Strategy**:

1. **Offline Detection**:
   - Check LLM API connectivity on startup (timeout after 2s)
   - If no config file or no API key: start in offline mode
   - If network unavailable: start in offline mode
   - Display banner: "⚠️ Offline Mode - Natural language disabled"

2. **Offline Capabilities** (always available):
   - Repository state display (all git read operations via local commands)
   - Direct git command input and execution
   - Command history and output display
   - All UI features except natural language translation

3. **Online-Only Features** (disabled offline):
   - Natural language to git command translation
   - Command suggestions based on context
   - Error message simplification (show raw git errors instead)

4. **Mode Switching**:
   - User can manually trigger connection retry: Press 'R' to reconnect
   - If connection established, switch to online mode automatically
   - If online operation fails, offer to switch to offline mode

5. **User Experience**:
   - Input widget changes prompt: "Enter git command:" (offline) vs "Natural language or git command:" (online)
   - Clear visual indicator in status bar: [OFFLINE] or [ONLINE]
   - Help text shows available features based on mode

## Assumptions
- Users have Git 2.20+ installed and accessible via PATH
- Users have access to LLM API (Claude API key or other configured provider) OR are willing to use offline mode
- Users are running on Unix-like systems (Linux, macOS) initially
- Terminal supports basic ANSI color codes and cursor control
- Users understand basic git concepts (what branches, commits, staging are)

## Solution Approaches

### Approach 1: Ratatui-based TUI with Direct LLM Integration (Recommended)
**Description**: Build the TUI using ratatui (mature Rust TUI library), integrate directly with LLM APIs via HTTP client, use libgit2 or git2-rs for some operations but primarily shell out to git commands.

**Pros**:
- Ratatui is well-maintained, feature-rich, and has good documentation
- Direct API integration gives full control over requests/responses
- Shelling out to git ensures compatibility with all git features
- Can parse git output in various formats (porcelain, etc.)
- Straightforward architecture

**Cons**:
- Need to implement LLM API clients for each provider
- Parsing git output can be brittle across versions
- Shell execution has security considerations

**Estimated Complexity**: Medium
**Risk Level**: Low

### Approach 2: Cursive-based TUI with LLM Abstraction Layer
**Description**: Use cursive TUI library with an abstraction layer for LLM providers (similar to langchain).

**Pros**:
- Cursive has different event handling model, might be simpler for some interactions
- Abstraction layer makes adding new LLM providers easier

**Cons**:
- Cursive is less actively maintained than ratatui
- Additional abstraction layer adds complexity
- More dependencies to manage

**Estimated Complexity**: Medium-High
**Risk Level**: Medium

### Approach 3: Embedded Git via libgit2 (git2-rs)
**Description**: Use libgit2 bindings instead of shelling out to git commands.

**Pros**:
- No shell execution security concerns
- Potentially more reliable than parsing text output
- Programmatic access to git internals

**Cons**:
- libgit2 doesn't support all git features
- More complex to implement
- Harder to stay in sync with git's evolution
- LLM would need to generate Rust API calls instead of git commands (doesn't align with learning goal)

**Estimated Complexity**: High
**Risk Level**: High

**Selected Approach**: Approach 1 (Ratatui + Direct LLM Integration)

## Open Questions

### Critical (Blocks Progress)
- None identified

### Important (Affects Design)
- [ ] Should the system support shell history/recall of previous natural language queries?
- [x] **Multi-step operations**: V1 will support single git commands only. Multi-step operations will be a future enhancement.
- [x] **Command sequences**: V1 presents one command at a time for confirmation.
- [x] **Command chaining**: Simple git command sequences using `&&` are allowed when they represent a single logical operation (e.g., `git add -A && git commit -m "message"`). Complex workflows requiring multiple user confirmations are V2.
- [x] **LLM Context Strategy**: Detailed escalation rules defined - see "LLM Context Strategy" section below.
- [x] **Iterative Clarification**: V2 feature - see "Iterative Clarification Flow (V2)" section below.

### Nice-to-Know (Optimization)
- [ ] Should the system cache common translations to reduce LLM API calls?
- [ ] Would users want keyboard shortcuts for common operations alongside natural language?
- [ ] Should there be a "teach me" mode that explains why certain git commands were chosen?
- [ ] **Query Classification**: How does the system determine which context escalation rule to apply for a given natural language query? (e.g., detecting commit vs branch vs diff operations). This affects LLM context strategy implementation.

## LLM Context Strategy

**Purpose**: Manage token usage and API costs while providing sufficient context for accurate command translation.

**Default Context** (sent with every query, ~500 tokens):
- Current branch name and upstream tracking info
- Ahead/behind commit counts
- File counts by category:
  - Untracked files: N (list first 10 if N ≤ 20)
  - Unstaged changes: M (list first 10 if M ≤ 20)
  - Staged changes: S (list all if S ≤ 20)
- Recent commit hashes and one-line messages (last 5)
- Repository state flags: clean/dirty, merge in progress, rebase in progress, detached HEAD
- Current stash count (if > 0)

**Context Escalation Rules** (operation-specific additional context):

1. **Commit Message Generation**:
   - Add: `git diff --stat` output (file list with +/- counts)
   - Add: First 20 lines of `git diff --staged` for context
   - Token budget: +1000 tokens

2. **Branch Operations** (create, switch, delete, merge):
   - Add: `git branch -a` output (all local and remote branches)
   - Add: Branch tracking info (`git branch -vv`)
   - Token budget: +300 tokens

3. **Merge/Rebase Conflict Resolution**:
   - Add: List of conflicted files
   - Add: Conflict markers for first conflicted file (full content)
   - Add: Brief conflict context (git status porcelain)
   - Token budget: +2000 tokens

4. **History Queries** ("show me commits", "what changed in last week"):
   - Add: Expanded `git log` output (format: hash, author, date, message)
   - Limit: Last 50 commits or date range specified
   - Token budget: +1500 tokens

5. **Diff/Change Queries** ("what changed in file X"):
   - Add: `git diff` output for specific file or paths
   - Limit: First 100 lines of diff
   - Token budget: +2000 tokens

6. **Stash Operations**:
   - Add: `git stash list` output
   - Add: `git stash show -p stash@{0}` summary
   - Token budget: +800 tokens

**Token Budget Cap**: Maximum 5000 tokens per request
- If context exceeds cap, truncate oldest/least relevant information
- Prioritize: current state > recent changes > history

**Optimization Strategies**:
- Use git porcelain formats for machine-readable output
- Compress file lists: "20 untracked files" instead of listing all
- Truncate large diffs with "... (500 more lines)" indicators
- Cache repository metadata (branch list, recent commits) for 30 seconds to avoid re-sending

## Iterative Clarification Flow (V2)

**Purpose**: Allow users to provide additional context when LLM-generated commands fail, enabling conversational refinement of git commands.

**Current V1 Flow** (for comparison):
```
Input → Translating → Preview → Execute → ShowingOutput → Input
                              ↓
                           Cancel → Input
```

**Enhanced V2 Flow with Clarification**:
```
Input → Translating → Preview → Execute → ShowingOutput
                              ↓                    ↓
                           Cancel              (if error)
                              ↓                    ↓
                            Input ←──────────  [Clarify?]
                                                   ↓ (press 'c')
                                              Clarifying
                                                   ↓
                                         Translating (with context)
                                                   ↓
                                               Preview
```

**State Machine Changes**:
1. **New State: `Clarifying`**
   - Activates when user presses 'c' after a failed command execution
   - Input widget prompts: "Provide clarification or additional context:"
   - User can type natural language explanation of what went wrong or what they meant
   - Pressing Enter sends clarification to LLM with enhanced context

2. **Enhanced Error Context**:
   - `GitCommand` struct extended to optionally store:
     - Previous command that failed
     - Git error message
     - User's clarification text
   - Context builder creates enriched prompt for LLM

3. **UI Components**:
   - **Error Display Widget** (enhanced):
     - Shows failed command
     - Shows git error message (translated to plain language)
     - Shows available actions: `[Any key: dismiss] [c: clarify] [q: quit]`
   - **Clarification Input Widget**:
     - Similar to natural language input widget
     - Different prompt to indicate it's clarification mode
     - Shows previous command and error above input area for reference

**LLM Context Enhancement for Clarification**:

When user provides clarification, the LLM receives:
```
Repository Context:
<standard repository context>

Previous Attempt:
User query: "add input.rs"
Generated command: git add input.rs
Error: fatal: pathspec 'input.rs' did not match any files

Repository Files:
<full file list showing available paths>

User Clarification: "I meant the file in src/ui"

Your task: Generate a corrected git command based on:
1. The original user intent
2. The error that occurred
3. The user's clarification
4. The actual repository file structure
```

**Example User Flow**:

1. **Initial Query**: User types "add input.rs"
2. **LLM Response**: `git add input.rs`
3. **User Confirms**: Presses Enter
4. **Execution Fails**: Error: `fatal: pathspec 'input.rs' did not match any files`
5. **Error Display**:
   ```
   Command Output
   ──────────────────────────────────────────────
   Command: git add input.rs
   Status: ✗ Failed (exit code: 1)

   Errors:
   Execution error: Git command failed: Command 'git add input.rs'
   failed with exit code 128: fatal: pathspec 'input.rs' did not
   match any files

   [Any key: dismiss] [c: clarify] [q: quit]
   ```
6. **User Presses 'c'**: Enters clarification mode
7. **Clarification Input**:
   ```
   Previous Command: git add input.rs
   Error: pathspec 'input.rs' did not match any files
   ──────────────────────────────────────────────
   Clarification: I meant the file in src/ui_
   ```
8. **User Presses Enter**: LLM re-translation with full context
9. **New Command**: `git add src/ui/input.rs`
10. **Success**: User confirms and executes successfully

**Implementation Components**:

1. **State Machine** (`src/ui/app.rs`):
   - Add `AppState::Clarifying` variant
   - Handle 'c' key in `ShowingOutput` state when error occurred
   - Transition: `ShowingOutput` (error) → `Clarifying` → `Translating` → `Preview`

2. **Context Builder** (`src/llm/context.rs`):
   - New method: `build_clarification_context(previous_command, error, clarification)`
   - Combines standard context + error context + user clarification
   - Special formatting to highlight the clarification request

3. **GitCommand Enhancement** (`src/llm/client.rs`):
   ```rust
   pub struct GitCommand {
       pub command: String,
       pub explanation: Option<String>,
       pub context: Option<CommandContext>,  // New field
   }

   pub struct CommandContext {
       pub previous_command: Option<String>,
       pub error_message: Option<String>,
       pub clarification: Option<String>,
   }
   ```

4. **UI Components**:
   - Modify `OutputDisplay` widget to detect errors and show clarification prompt
   - Create clarification input mode (can reuse `InputWidget` with different prompt)
   - Add keyboard handler for 'c' key in error state

**Benefits**:
- Conversational refinement reduces user frustration
- LLM learns from its mistakes within a session
- Users don't need to retype entire queries
- Natural way to handle ambiguous file paths, branch names, etc.

**Limitations** (V2 scope):
- Single clarification iteration (no multi-turn conversation)
- Clarification context not persisted across sessions
- No learning/improvement of base model (each session starts fresh)

**Future Enhancements** (V3+):
- Multi-turn clarification conversations
- Session history for context across multiple commands
- Learning from user corrections to improve future suggestions
- Suggested clarification questions from LLM ("Did you mean: src/ui/input.rs?")

## Performance Requirements

**Baseline Repository Assumptions**:
- 1000 files tracked
- 1000 commits in history
- Working directory with ~100 modified files
- Standard SSD storage

**Performance Targets**:
- **UI Refresh Rate**: <100ms for repository state updates (cold start from disk)
- **LLM Response Time**: Display "thinking" indicator, acceptable up to 5 seconds for translation
- **Command Execution**: Show progress for long-running operations (push, pull, fetch)
- **Startup Time**: <500ms to launch and display repository state (warm start, config cached)
- **Memory Usage**: <100MB for typical operation
- **Large Repository Handling**: For repos >10K files, implement lazy loading and pagination

## Security Considerations
- **Command Injection**: Must sanitize any user input before shell execution
- **Dangerous Operations**: Force push, hard reset, filter-branch, clean -fd require explicit confirmation (user must type "CONFIRM")
- **API Keys**: LLM API keys must be stored securely (env vars or secure config file with restricted permissions 600)
- **Repository Access**: Operate only within detected git repository boundaries
- **LLM Output Validation**:
  - **Allowed Git Subcommands** (V1 allowlist):
    - Read operations: `status`, `log`, `show`, `diff`, `branch`, `tag`, `remote`, `reflog`, `blame`, `describe`
    - Write operations: `add`, `commit`, `checkout`, `switch`, `restore`, `reset`, `revert`, `merge`, `rebase`, `cherry-pick`, `stash`, `clean`
    - Remote operations: `push`, `pull`, `fetch`, `clone`
    - Branch operations: `branch` (with create/delete flags), `merge`, `rebase`
    - Configuration: `config` (limited to repo-level only, no global/system)
  - **Command Structure Validation**:
    - Use regex to verify: `^git\s+(subcommand)\s+[flags and args]$`
    - Allow `&&` only between git commands for single logical operations
    - Reject commands with suspicious operators: `;`, `||`, `|` (pipe), `>`, `<`, `$()`, backticks
    - Reject git commands with `--exec` or `-c core.sshCommand` flags (arbitrary code execution vectors)
  - **Present ALL commands to user for review before execution**
- **Audit Trail**: Log all executed commands to `~/.config/gitalky/history.log` with timestamps

## Test Scenarios
### Functional Tests
1. **Basic Workflow**
   - User: "show me what changed"
   - System: Displays current status in panel, suggests `git status` or `git diff`
   - Execute and display results

2. **Commit Creation**
   - User: "commit these changes with message 'fix bug'"
   - System: Suggests `git commit -m "fix bug"` (or with -a if appropriate)
   - User confirms, commit is created, UI refreshes

3. **Branch Operations**
   - User: "create a new branch called feature-x from main"
   - System: Suggests `git checkout -b feature-x main`
   - User confirms, branch created and checked out

4. **History Viewing**
   - User: "show me the last 10 commits"
   - System: Suggests `git log -10 --oneline` or similar
   - Results displayed in panel

5. **Ambiguous Query**
   - User: "undo"
   - System: Asks for clarification via LLM or suggests multiple options
   - User selects appropriate action

6. **Error Handling**
   - User: "push to origin"
   - Git error: "no upstream branch"
   - System: Translates to "No remote branch is set up. Do you want to push and set up tracking?"

7. **Error Recovery Patterns**
   - **Scenario A - No upstream**: `git push` fails with no upstream configured
     - System translates error to plain language
     - System suggests: `git push -u origin <branch-name>`
     - User confirms and executes recovery command
   - **Scenario B - Merge conflict**: `git merge` results in conflicts
     - System detects conflict state
     - System explains: "Merge has conflicts in 3 files. You can: [1] View conflicts [2] Abort merge [3] Enter git command directly"
     - User selects option, system translates to appropriate command
   - **Scenario C - Detached HEAD**: Repository is in detached HEAD state
     - System shows warning banner in repo state panel
     - Natural language suggestions context-aware: "create branch here" → `git checkout -b <name>`
   - **Scenario D - Unfinished operation**: Rebase/merge/cherry-pick in progress
     - System shows operation status in repo panel
     - Suggests: "continue the operation" / "abort the operation"

8. **Dangerous Operation**
   - User: "force push to main"
   - System: Shows extra warning, requires typing "CONFIRM" or similar

9. **Edit Proposed Command**
   - System suggests: `git commit -m "message"`
   - User edits to: `git commit -m "better message"`
   - Edited command executes

### Non-Functional Tests
1. **Performance**: Repository with 1000+ files should still refresh UI in <100ms
2. **API Failure - Online Mode**:
   - Disconnect network during operation
   - System shows: "LLM API unavailable. [1] Retry [2] Enter git command directly [3] Switch to offline mode"
   - User selects option 3
   - System disables natural language input, continues showing repo state
3. **Offline Mode Behavior**:
   - Launch gitalky without network connection
   - System detects offline state, shows warning banner
   - Repository state panel works normally (read-only git operations)
   - Natural language input disabled with message: "Offline mode - enter git commands directly or connect to enable natural language"
   - User can type raw git commands for execution
4. **Malformed LLM Output**: If LLM returns invalid command, show error and allow retry
5. **Large Diff**: Repository with large changes should handle gracefully (paginate or summarize)

## Architecture Overview

### Component Structure
```
gitalky/
├── src/
│   ├── main.rs                 # Entry point, TUI setup
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── app.rs              # Main app state and event loop
│   │   ├── repo_panel.rs       # Repository state display (Magit-style)
│   │   ├── input.rs            # Natural language input widget
│   │   ├── command_preview.rs  # Git command confirmation widget
│   │   └── output.rs           # Command output display
│   ├── git/
│   │   ├── mod.rs
│   │   ├── repository.rs       # Git repo detection and state queries
│   │   ├── executor.rs         # Git command execution
│   │   └── parser.rs           # Parse git command output
│   ├── llm/
│   │   ├── mod.rs
│   │   ├── client.rs           # LLM API client trait
│   │   ├── anthropic.rs        # Claude API implementation
│   │   ├── openai.rs           # OpenAI API implementation (future)
│   │   └── translator.rs       # Natural language to git command translation
│   ├── config/
│   │   ├── mod.rs
│   │   └── settings.rs         # Configuration file handling
│   └── error.rs                # Error types and handling
```

### Data Flow
1. User enters natural language query in input widget
2. Query sent to LLM translator with repository context
3. LLM returns proposed git command(s)
4. Command displayed in preview widget for confirmation
5. User approves/edits/cancels
6. If approved, command executed via git executor
7. Output captured and displayed
8. Repository state refreshed
9. UI updates to show new state

### Repository State Panel (Magit-inspired)
Display sections:
- **Head**: Current branch, upstream, ahead/behind count
- **Untracked files**: Files not in git
- **Unstaged changes**: Modified files not staged
- **Staged changes**: Files ready to commit
- **Stashes**: List of stashes (if any)
- **Recent commits**: Last 5-10 commits with hash and message

Each section collapsible/expandable, items actionable (though in v1, primary interaction is through natural language).

## Dependencies
- **External Libraries**:
  - `ratatui` - Terminal UI framework
  - `crossterm` - Terminal manipulation (backend for ratatui)
  - `tokio` - Async runtime for API calls
  - `reqwest` - HTTP client for LLM APIs
  - `serde` - Serialization for config and API
  - `serde_json` - JSON handling
  - `toml` - Config file parsing

- **External Services**:
  - Anthropic Claude API (default)
  - OpenAI API (future)
  - Local LLM APIs like Ollama (future)

- **System Requirements**:
  - Git 2.20+ installed (for reliable porcelain format support)
  - Internet connection for LLM API (optional - offline mode available)

## Configuration File Design
Location: `~/.config/gitalky/config.toml`

```toml
[llm]
provider = "anthropic"  # or "openai", "ollama"
model = "claude-sonnet-4.5-20250929"
api_key_env = "ANTHROPIC_API_KEY"  # env var name, not the key itself

[ui]
theme = "dark"  # or "light"
refresh_interval_ms = 100
show_help_hints = true

[behavior]
auto_refresh = true
confirm_dangerous = true
dangerous_operations = ["push --force", "reset --hard", "clean -fd", "filter-branch"]
show_raw_errors = false  # default to simplified errors
enable_command_logging = true  # log to history.log

[git]
# Future: custom git binary path, etc.
```

## First-Run Setup Flow

**Initial Launch Sequence** (when no config file exists):

1. **Welcome Screen**:
   ```
   ╔═══════════════════════════════════════════════════════════╗
   ║                  Welcome to Gitalky!                      ║
   ║                                                           ║
   ║  A natural language interface for Git                    ║
   ║                                                           ║
   ║  To use natural language features, configure an LLM:     ║
   ║                                                           ║
   ║  [1] Anthropic Claude (recommended)                      ║
   ║  [2] OpenAI GPT (coming soon)                            ║
   ║  [3] Local/Ollama (coming soon)                          ║
   ║  [4] Skip - Use offline mode (git commands only)         ║
   ║                                                           ║
   ║  Select option (1-4):                                    ║
   ╚═══════════════════════════════════════════════════════════╝
   ```

2. **If Provider Selected** (options 1-3):
   - Prompt: "Enter API key (or press Enter to use environment variable):"
   - If user enters key: Store in config file with 600 permissions
   - If user presses Enter: Prompt for environment variable name (default: ANTHROPIC_API_KEY)
   - Validate API key by making test request (timeout 5s)
   - If validation fails: Offer to retry or skip to offline mode

3. **If Skip Selected** (option 4):
   - Show message: "Starting in offline mode. You can configure LLM later in ~/.config/gitalky/config.toml"
   - Create config file with offline defaults

4. **Config File Creation**:
   - Create directory: `~/.config/gitalky/` if not exists
   - Create file: `config.toml` with user selections
   - Set file permissions: 600 (owner read/write only)
   - Create empty history log: `history.log`

5. **Launch Main TUI**:
   - Show brief help overlay on first run: "Press ? for help, Esc to dismiss"
   - Proceed to main application interface

**Reconfiguration**: Users can delete config file or edit it directly to trigger re-setup or change providers

## Risks and Mitigation
| Risk | Probability | Impact | Mitigation Strategy |
|------|------------|--------|-------------------|
| LLM returns invalid/dangerous commands | Medium | High | Validate output, require confirmation, maintain allowlist of safe command patterns |
| API rate limits/costs | Medium | Medium | Implement request caching, allow local LLM configuration, add rate limiting |
| Git output parsing breaks across versions | Medium | Medium | Use git's porcelain formats where available, test across git versions, graceful degradation |
| UI performance with large repos | Low | Medium | Lazy loading, pagination, limit displayed items, async loading |
| Command injection vulnerabilities | Low | Critical | Strict input validation, avoid shell interpolation, use proper escaping |
| LLM hallucinations (making up git flags) | Medium | Medium | Validate commands against known git syntax, allow user review before execution |

## UI/UX Wireframe (Text Representation)

```
┌─────────────────────────────────────────────────────────────────┐
│ Gitalky - /home/user/project                        [? for help]│
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
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
│ Recent commits                                                   │
│   abc123 (HEAD -> main) Add config module                       │
│   def456 Initial commit                                         │
│                                                                   │
├─────────────────────────────────────────────────────────────────┤
│ Natural Language Query:                                          │
│ > commit all my changes with a meaningful message_              │
│                                                                   │
├─────────────────────────────────────────────────────────────────┤
│ Proposed Command:                                                │
│ $ git add -A && git commit -m "Update main.rs and README.md,   │
│   add config module"                                             │
│                                                                   │
│ [Enter] Execute  [E] Edit  [Esc] Cancel                         │
└─────────────────────────────────────────────────────────────────┘
```

## Implementation Phases (Preview)
This section provides a rough breakdown for planning purposes:

1. **Core TUI & Git Integration**: Basic ratatui app, repository state display, git command execution
2. **LLM Integration**: Claude API client, natural language to command translation
3. **Command Preview & Execution**: Confirmation flow, command editing, execution with output
4. **Error Handling & Safety**: Error translation, dangerous operation detection and confirmation
5. **Configuration System**: Config file parsing, LLM provider selection, user preferences
6. **Polish & Documentation**: Help system, keyboard shortcuts, user documentation

## Success Metrics
- User can complete common git workflows (clone, branch, commit, push, pull) using only natural language
- Command translation accuracy >90% for common operations
- Users report increased understanding of git commands through the translation visibility
- Average time to complete git operations comparable to or faster than manual command entry
- Zero command injection vulnerabilities in security review

## References
- Magit interface design: https://magit.vc/
- Ratatui documentation: https://ratatui.rs/
- Git command documentation: https://git-scm.com/docs
- Anthropic Claude API: https://docs.anthropic.com/

## Notes
- The Magit-inspired interface is key to the UX - every visible piece of information should eventually be actionable
- Transparency is a core value - users should always see what git commands are being run
- This is both a productivity tool and a learning tool
- Future versions could support plugins for custom command translations or workflows
- Consider adding telemetry (opt-in) to improve translation quality over time

## Self-Review Notes

**First Self-Review - Date**: 2025-09-30

**Improvements Made During First Self-Review**:

1. **Resolved Open Questions**:
   - Clarified that V1 will handle single commands only (no multi-step operations)
   - Defined LLM context strategy: send summary info, not full diffs, to manage tokens/costs

2. **Enhanced Security Section**:
   - Added specific command validation approach (allowlist + regex patterns)
   - Specified dangerous operation confirmation mechanism (type "CONFIRM")
   - Added audit trail logging to history.log
   - Detailed shell operator handling

3. **Improved Configuration**:
   - Added command logging toggle to config
   - Documented default behavior when no config exists
   - Specified first-run setup flow
   - Added read-only fallback mode if no API key

**Second Self-Review - Date**: 2025-10-04

**Additional Improvements Made**:

1. **LLM Context Strategy** (new section added):
   - Detailed default context specification (~500 tokens)
   - 6 operation-specific escalation rules with token budgets
   - 5000 token cap with prioritization strategy
   - Optimization strategies for cost management

2. **Command Validation Allowlist** (security section enhanced):
   - Complete allowlist of git subcommands by category
   - Identified dangerous flags (`--exec`, `-c core.sshCommand`)
   - Specific command structure validation rules

3. **Command Chaining Clarification**:
   - Defined allowed use of `&&` for single logical operations
   - Clarified V2 boundary for multi-step workflows

4. **Error Recovery Patterns** (test scenarios enhanced):
   - 4 detailed recovery scenarios with user flows
   - Context-aware suggestions for error states

5. **Offline Mode** (new section added):
   - Complete graceful degradation strategy
   - Offline detection with 2s timeout
   - Clear capability boundaries and mode switching UX

6. **Performance Requirements Context**:
   - Added baseline repository size assumptions
   - Clarified cold/warm start distinctions
   - Large repository handling strategy

7. **First-Run Setup Flow** (new section added):
   - Visual welcome screen wireframe
   - Complete step-by-step configuration process
   - API key validation flow with fallbacks

**Final Assessment**:
- **Completeness**: 9.5/10 - All major areas comprehensively covered
- **Clarity**: 9/10 - Precise, unambiguous technical details
- **Implementability**: 9.5/10 - Clear path forward with actionable requirements
- **Security**: 9/10 - Comprehensive validation and audit strategy

**Confidence Level**: Very High (95%) - Specification is ready for planning phase

**Status**: ✅ Ready to proceed to SPIDER-SOLO Planning phase

## Approval
- [ ] Technical Lead Review
- [ ] Product Owner Review
- [ ] Stakeholder Sign-off