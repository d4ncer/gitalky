# Gitalky

**Natural Language Git Terminal UI**

Talk to git in plain English. Gitalky translates your natural language requests into git commands, shows you what it will do, and lets you approve before execution.

```bash
# Instead of remembering:
git checkout -b feature-x && git push -u origin feature-x

# Just say:
"create a new branch called feature-x and push it to remote"
```

## Quick Start

### Prerequisites

- Git 2.20 or higher
- Rust 1.90+ (for building from source)
- Anthropic API key (optional - works offline without one)

### Installation

```bash
# Clone and build
git clone https://github.com/yourusername/gitalky.git
cd gitalky
cargo build --release

# Run
cargo run --release
```

### First Run

On first launch, Gitalky will guide you through setup:

1. **Choose your mode:**
   - Anthropic Claude (AI-powered natural language)
   - Offline mode (direct git commands only)

2. **Configure API key (if using AI):**
   - Environment variable (recommended): `export ANTHROPIC_API_KEY='your-key'`
   - Or store in config file (less secure but convenient)

3. **Start using Gitalky!**

## Usage

### Natural Language Mode

Type what you want to do in plain English:

```
"show me what changed"
"commit these changes with message 'fix bug'"
"create a new branch called feature-x from main"
"show me the last 10 commits"
"stage all modified files"
"undo the last commit but keep the changes"
```

### Offline Mode

Enter git commands directly (without the `git` prefix):

```
status
add .
commit -m "message"
```

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Enter` | Submit query / Execute command |
| `e` | Edit proposed command |
| `Esc` | Cancel current operation |
| `?` | Show help |
| `r` | Retry LLM connection (when offline) |
| `t` | Toggle raw/simplified error display |
| `q` | Quit |

## Features

### 🤖 AI-Powered Translation
- Converts natural language to git commands using Claude
- Shows you exactly what will be executed
- Edit commands before execution

### 🔒 Safety First
- All commands require your approval
- Dangerous operations (force push, hard reset) require typing "CONFIRM"
- Command validation prevents injection attacks
- Audit logging of all executed commands

### 📊 Live Repository View
Magit-inspired interface showing:
- Current branch and upstream tracking
- Staged/unstaged changes
- Untracked files
- Stashes
- Recent commits

### 💡 User-Friendly Errors
- Git errors translated to plain language
- Actionable suggestions for common issues
- Raw error available on demand

### 🔌 Works Offline
- No API key? No problem
- Falls back to direct git command mode
- Reconnect anytime with `r` key

## Configuration

Config file: `~/.config/gitalky/config.toml`

```toml
[llm]
provider = "anthropic"
model = "claude-sonnet-4-20250514"
api_key_env = "ANTHROPIC_API_KEY"

[ui]
refresh_interval_ms = 100
max_commits_display = 5
max_stashes_display = 5

[behavior]
auto_refresh = true
confirm_dangerous_ops = true
log_commands = true

[git]
timeout_seconds = 30
```

### Environment Variables

- `ANTHROPIC_API_KEY` - Your Anthropic API key (recommended)
- `HOME` - Used to locate config directory

### Audit Log

All executed commands are logged to: `~/.config/gitalky/history.log`

Format: `[timestamp] [repo_path] [command] [exit_code]`

## Examples

### Common Workflows

**Starting a new feature:**
```
> "create a branch called feature-auth from main"
→ git checkout -b feature-auth main
[Enter to execute]
```

**Reviewing changes:**
```
> "show me what I changed in the last hour"
→ git log --since="1 hour ago" -p
[Enter to execute]
```

**Fixing mistakes:**
```
> "undo the last commit but keep my changes"
→ git reset --soft HEAD~1
[Enter to execute]
```

**Cleaning up:**
```
> "delete all local branches that are merged"
→ git branch --merged | grep -v "\*" | xargs git branch -d
⚠️  DANGEROUS OPERATION - Type CONFIRM to execute
```

## Troubleshooting

### First-Run Wizard Doesn't Appear
- Delete `~/.config/gitalky/config.toml` to trigger setup again

### "Not a git repository" Error
- Run gitalky from within a git repository
- Or `cd` to your project first

### API Connection Failed
- Check your API key: `echo $ANTHROPIC_API_KEY`
- Verify network connectivity
- Press `r` to retry connection
- Or use offline mode (press `4` during setup)

### Commands Are Rejected
- Check the audit log for details: `~/.config/gitalky/history.log`
- Some commands are blocked for security (pipes, redirects, etc.)
- Edit the proposed command with `e` if needed

## Architecture

Gitalky follows the **Codev** methodology with the **SPIDER-SOLO** protocol:

```
codev/
├── specs/     # Feature specifications
├── plans/     # Implementation plans
├── reviews/   # Post-implementation reviews
└── protocols/ # Development protocols
```

See [codev/specs/0002-natural-language-git-tui.md](codev/specs/0002-natural-language-git-tui.md) for the complete specification.

## Development

### Building from Source

```bash
cargo build
cargo test
cargo run
```

### Project Structure

```
src/
├── main.rs              # Entry point
├── config/              # Configuration & first-run wizard
├── git/                 # Git operations
│   ├── executor.rs      # Command execution
│   ├── parser.rs        # Output parsing
│   └── repository.rs    # Repository state
├── llm/                 # LLM integration
│   ├── client.rs        # LLM client trait
│   ├── anthropic.rs     # Claude implementation
│   ├── translator.rs    # NL → git translation
│   └── context.rs       # Context building
├── security/            # Security & validation
│   └── validator.rs     # Command validation
├── error_translation/   # Error translation
├── audit/               # Command logging
└── ui/                  # Terminal UI
    ├── app.rs           # Main app state
    ├── repo_panel.rs    # Repository display
    ├── input.rs         # Input widget
    ├── command_preview.rs # Command review
    ├── output.rs        # Output display
    └── help.rs          # Help screen
```

### Running Tests

```bash
# All tests
cargo test

# Unit tests only
cargo test --lib

# Integration tests
cargo test --test integration_test

# Specific test
cargo test test_command_validation
```

## Security

### Command Validation

Gitalky validates all commands against:
- Allowlist of safe git subcommands
- Injection attack patterns (`;`, `|`, `>`, `$()`, etc.)
- Dangerous operation detection

### API Key Storage

**Recommended:** Use environment variables
```bash
export ANTHROPIC_API_KEY='your-key-here'
```

**Alternative:** Config file (permissions set to 600)
```toml
[llm]
api_key = "your-key-here"  # Only if not using env var
```

### Audit Trail

All commands logged to `~/.config/gitalky/history.log`:
```
[2025-10-07 10:30:15] [/home/user/project] git status [exit: 0]
[2025-10-07 10:30:42] git commit -m "fix bug" [exit: 0]
[2025-10-07 10:31:05] git push --force [exit: 1]  # User confirmed dangerous op
```

## Roadmap

### v1.0 (Current)
- ✅ Natural language translation (Anthropic Claude)
- ✅ TUI with live repository view
- ✅ Command validation and safety
- ✅ Error translation
- ✅ Offline mode
- ✅ Audit logging

### Future Versions
- Multi-step workflows (interactive refinement)
- Additional LLM providers (OpenAI, Ollama)
- Windows support
- Command history and suggestions
- Git hooks integration
- Custom command templates

## Contributing

Gitalky uses the Codev methodology. To contribute:

1. Check existing specs in `codev/specs/`
2. Create a spec for your feature
3. Get approval before implementation
4. Follow the SPIDER-SOLO protocol phases
5. Submit PR with spec + implementation + review

See [CLAUDE.md](CLAUDE.md) for detailed development guidelines.

## License

[Your License Here]

## Acknowledgments

- Built with [Ratatui](https://github.com/ratatui-org/ratatui) for the TUI
- Powered by [Anthropic Claude](https://www.anthropic.com/claude) for natural language understanding
- Inspired by [Magit](https://magit.vc/) for the repository UI design

---

**Need help?** Press `?` in the app or open an issue on GitHub.
