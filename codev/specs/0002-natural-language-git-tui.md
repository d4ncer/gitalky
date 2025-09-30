# Specification: Natural Language Git Terminal UI

## Metadata
- **ID**: spec-2025-09-30-natural-language-git-tui
- **Status**: draft
- **Created**: 2025-09-30

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

## Constraints
### Technical Constraints
- Rust programming language (edition 2024)
- Must work in standard terminal emulators
- Must handle git repositories with various states (clean, dirty, conflicts, detached HEAD, etc.)
- LLM API calls introduce latency - UI must remain responsive
- Must parse git command output reliably

### Business Constraints
- LLM API usage incurs costs - should be mindful of token usage
- Must work offline for local operations (status display)
- First version should be fully functional end-to-end

## Assumptions
- Users have git installed and accessible via PATH
- Users have access to LLM API (Claude API key or other configured provider)
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
- [ ] How should the system handle multi-step operations (e.g., "create branch and cherry-pick these commits")?
- [ ] Should the command confirmation step allow multiple commands if LLM suggests a sequence?
- [ ] What level of git context should be sent to LLM (full diff, just file names, etc.)?

### Nice-to-Know (Optimization)
- [ ] Should the system cache common translations to reduce LLM API calls?
- [ ] Would users want keyboard shortcuts for common operations alongside natural language?
- [ ] Should there be a "teach me" mode that explains why certain git commands were chosen?

## Performance Requirements
- **UI Refresh Rate**: <100ms for repository state updates
- **LLM Response Time**: Display "thinking" indicator, acceptable up to 5 seconds for translation
- **Command Execution**: Show progress for long-running operations (push, pull, fetch)
- **Startup Time**: <500ms to launch and display repository state
- **Memory Usage**: <100MB for typical operation

## Security Considerations
- **Command Injection**: Must sanitize any user input before shell execution
- **Dangerous Operations**: Force push, hard reset, filter-branch, clean -fd require explicit confirmation
- **API Keys**: LLM API keys must be stored securely (env vars or secure config file with restricted permissions)
- **Repository Access**: Operate only within detected git repository boundaries
- **LLM Output Validation**: Validate that LLM output is actually a valid git command before presenting to user
- **Audit Trail**: Consider logging executed commands for security review

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

7. **Dangerous Operation**
   - User: "force push to main"
   - System: Shows extra warning, requires typing "CONFIRM" or similar

8. **Edit Proposed Command**
   - System suggests: `git commit -m "message"`
   - User edits to: `git commit -m "better message"`
   - Edited command executes

### Non-Functional Tests
1. **Performance**: Repository with 1000+ files should still refresh UI in <100ms
2. **API Failure**: If LLM API is unavailable, show clear error and allow user to input git command directly
3. **Malformed LLM Output**: If LLM returns invalid command, show error and allow retry
4. **Large Diff**: Repository with large changes should handle gracefully (paginate or summarize)

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
  - Git 2.0+ installed
  - Internet connection for LLM API

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

[git]
# Future: custom git binary path, etc.
```

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
(To be added after initial self-review)

## Approval
- [ ] Technical Lead Review
- [ ] Product Owner Review
- [ ] Stakeholder Sign-off