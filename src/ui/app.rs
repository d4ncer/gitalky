use crate::error::Result;
use crate::error_translation::ErrorTranslator;
use crate::git::{Repository, RepositoryState};
use crate::llm::{AnthropicClient, ContextBuilder, Translator};
use crate::security::CommandValidator;
use crate::ui::command_preview::CommandPreview;
use crate::ui::input::{InputMode, InputWidget};
use crate::ui::output::{CommandOutput, OutputDisplay};
use crate::ui::repo_panel::RepositoryPanel;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::env;
use std::io;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    Offline,
}

/// Application state for UI flow
#[derive(Debug, Clone, PartialEq)]
enum AppState {
    Input,              // User typing query
    Translating,        // Waiting for LLM response
    Preview,            // Showing proposed command
    ConfirmDangerous,   // Confirming dangerous operation
    Executing,          // Running command
    ShowingOutput,      // Displaying command output
}

/// Main application state
pub struct App {
    repo: Repository,
    repo_state: RepositoryState,
    should_quit: bool,
    mode: AppMode,
    state: AppState,

    // Widgets
    input: InputWidget,
    preview: Option<CommandPreview>,
    output: OutputDisplay,

    // LLM components
    translator: Option<Translator>,

    // Security
    validator: CommandValidator,

    // State management
    pending_query: Option<String>,
    error_message: Option<String>,
    dangerous_op_type: Option<crate::security::DangerousOp>,
    confirmation_input: String,
}

impl App {
    /// Create a new App instance with the given repository
    pub fn new(repo: Repository) -> Result<Self> {
        let repo_state = repo.state()?;

        // Try to initialize LLM translator
        let translator = Self::try_init_translator(&repo);
        let mode = if translator.is_some() {
            AppMode::Normal
        } else {
            AppMode::Offline
        };

        let input_mode = if mode == AppMode::Normal {
            InputMode::Online
        } else {
            InputMode::Offline
        };

        let mut input = InputWidget::new(input_mode);
        input.set_active(true); // Start with input focused

        Ok(Self {
            repo,
            repo_state,
            should_quit: false,
            mode,
            state: AppState::Input,
            input,
            preview: None,
            output: OutputDisplay::new(),
            translator,
            validator: CommandValidator::new(),
            pending_query: None,
            error_message: None,
            dangerous_op_type: None,
            confirmation_input: String::new(),
        })
    }

    /// Try to initialize translator with API key
    fn try_init_translator(repo: &Repository) -> Option<Translator> {
        if let Ok(api_key) = env::var("ANTHROPIC_API_KEY") {
            let client = Box::new(AnthropicClient::new(api_key));
            let context_builder = ContextBuilder::new(repo.clone());
            Some(Translator::new(client, context_builder))
        } else {
            None
        }
    }

    /// Run the application event loop (async)
    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        loop {
            terminal.draw(|f| self.render(f))?;

            // Poll for events with 100ms timeout for refresh
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key_event(key, terminal).await?;
                }
            } else {
                // Refresh repository state on timeout (only when not busy)
                if (self.state == AppState::Input || self.state == AppState::ShowingOutput)
                    && let Err(e) = self.refresh_repo_state()
                {
                    self.mode = AppMode::Offline;
                    eprintln!("Failed to refresh repo state: {}", e);
                }
            }

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    /// Render the UI
    fn render(&self, frame: &mut Frame) {
        // Clear the entire frame to prevent artifacts
        frame.render_widget(ratatui::widgets::Clear, frame.area());

        let size = frame.area();

        // Create layout: title bar + content + bottom panel + status
        // Adjust constraints based on state to give more room for preview/output
        let bottom_height = match self.state {
            AppState::Preview => 8,       // Command preview (removed control hints)
            AppState::ShowingOutput => 15, // Output needs more room
            _ => 3,                        // Input and loading are small
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),                    // Title
                Constraint::Min(5),                       // Content (repo panel)
                Constraint::Length(bottom_height),        // Input/Preview/Output
                Constraint::Length(1),                    // Status
            ])
            .split(size);

        // Title bar
        let title = format!(
            "Gitalky - {}{}",
            self.repo.path().display(),
            if self.mode == AppMode::Offline {
                " [OFFLINE]"
            } else {
                ""
            }
        );
        let title_block = Block::default()
            .title(title)
            .title_alignment(ratatui::layout::Alignment::Left)
            .borders(Borders::ALL);
        frame.render_widget(title_block, chunks[0]);

        // Repository panel
        let repo_panel = RepositoryPanel::new(&self.repo_state);
        frame.render_widget(repo_panel, chunks[1]);

        // Bottom section depends on state
        match self.state {
            AppState::Input => {
                frame.render_widget(&self.input, chunks[2]);
            }
            AppState::Translating => {
                let loading = Paragraph::new("⏳ Translating with Claude...")
                    .style(Style::default().fg(Color::Yellow))
                    .block(Block::default().borders(Borders::ALL));
                frame.render_widget(loading, chunks[2]);
            }
            AppState::Preview => {
                if let Some(ref preview) = self.preview {
                    frame.render_widget(preview, chunks[2]);
                }
            }
            AppState::ConfirmDangerous => {
                self.render_dangerous_confirmation(frame, chunks[2]);
            }
            AppState::Executing => {
                let executing = Paragraph::new("⚙️  Executing command...")
                    .style(Style::default().fg(Color::Cyan))
                    .block(Block::default().borders(Borders::ALL));
                frame.render_widget(executing, chunks[2]);
            }
            AppState::ShowingOutput => {
                frame.render_widget(&self.output, chunks[2]);
            }
        }

        // Status bar
        let status_text = match self.state {
            AppState::Input => "Enter: submit | q: quit",
            AppState::Translating => "Please wait...",
            AppState::Preview => "Enter: execute | E: edit | Esc: cancel",
            AppState::ConfirmDangerous => "Type CONFIRM to execute | Esc: cancel",
            AppState::Executing => "Please wait...",
            AppState::ShowingOutput => "Any key to continue | q: quit",
        };

        let status_style = if let Some(ref error) = self.error_message {
            let error_text = format!("Error: {} | Press any key", error);
            frame.render_widget(
                Paragraph::new(error_text).style(Style::default().fg(Color::Red)),
                chunks[3],
            );
            return;
        } else {
            Style::default()
        };

        frame.render_widget(Paragraph::new(status_text).style(status_style), chunks[3]);
    }

    /// Handle keyboard events
    async fn handle_key_event<B: Backend>(&mut self, key: KeyEvent, terminal: &mut Terminal<B>) -> io::Result<()> {
        // Only handle key press events (not release or repeat)
        if key.kind != KeyEventKind::Press {
            return Ok(());
        }

        // Clear error message on any key
        if self.error_message.is_some() {
            self.error_message = None;
            return Ok(());
        }

        // Global quit
        if matches!(key.code, KeyCode::Char('q') | KeyCode::Char('Q')) && self.state == AppState::Input {
            self.should_quit = true;
            return Ok(());
        }

        match self.state {
            AppState::Input => self.handle_input_state(key, terminal).await?,
            AppState::Preview => self.handle_preview_state(key, terminal).await?,
            AppState::ConfirmDangerous => self.handle_confirm_dangerous_state(key, terminal).await?,
            AppState::ShowingOutput => self.handle_output_state(key),
            AppState::Translating | AppState::Executing => {
                // No input allowed during these states
            }
        }
        Ok(())
    }

    async fn handle_input_state<B: Backend>(&mut self, key: KeyEvent, terminal: &mut Terminal<B>) -> io::Result<()> {
        match key.code {
            KeyCode::Enter => {
                let query = self.input.take_input().trim().to_string();
                if query.is_empty() {
                    return Ok(());
                }

                self.pending_query = Some(query.clone());

                // Check if it looks like a direct git command
                if query.starts_with("git ") || self.mode == AppMode::Offline {
                    // Direct command execution
                    let command = if query.starts_with("git ") {
                        query
                    } else {
                        format!("git {}", query)
                    };

                    self.preview = Some(CommandPreview::new(command, None));
                    self.state = AppState::Preview;
                } else {
                    // Translate with LLM - set state and redraw to show loading
                    self.state = AppState::Translating;
                    terminal.draw(|f| self.render(f))?;
                    self.translate_query(query).await;
                }
            }
            _ => {
                self.input.handle_key(key);
            }
        }
        Ok(())
    }

    async fn translate_query(&mut self, query: String) {
        if let Some(ref translator) = self.translator {
            match translator.translate(&query).await {
                Ok(git_command) => {
                    self.preview = Some(CommandPreview::new(
                        git_command.command,
                        git_command.explanation,
                    ));
                    self.state = AppState::Preview;
                }
                Err(e) => {
                    self.error_message = Some(format!("Translation failed: {}", e));
                    self.state = AppState::Input;
                }
            }
        } else {
            self.error_message = Some("LLM not available".to_string());
            self.state = AppState::Input;
        }
    }

    async fn handle_preview_state<B: Backend>(&mut self, key: KeyEvent, terminal: &mut Terminal<B>) -> io::Result<()> {
        if let Some(ref mut preview) = self.preview {
            if preview.is_edit_mode() {
                // In edit mode
                match key.code {
                    KeyCode::Enter => {
                        // Execute edited command
                        preview.exit_edit_mode();
                        self.execute_command(terminal).await?;
                    }
                    KeyCode::Esc => {
                        // Cancel editing, back to preview
                        preview.exit_edit_mode();
                    }
                    _ => {
                        preview.handle_key(key);
                    }
                }
            } else {
                // Normal preview mode
                match key.code {
                    KeyCode::Enter => {
                        // Execute command
                        self.execute_command(terminal).await?;
                    }
                    KeyCode::Char('e') | KeyCode::Char('E') => {
                        // Enter edit mode
                        preview.enter_edit_mode();
                    }
                    KeyCode::Esc => {
                        // Cancel, back to input
                        self.preview = None;
                        self.state = AppState::Input;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    async fn execute_command<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        if let Some(ref preview) = self.preview {
            let command = preview.get_command().to_string();

            // Validate command before execution
            match self.validator.validate(&command) {
                Ok(validated) => {
                    // Check if dangerous operation requires confirmation
                    if validated.is_dangerous {
                        // Transition to confirmation state
                        self.dangerous_op_type = validated.danger_type.clone();
                        self.confirmation_input.clear();
                        self.state = AppState::ConfirmDangerous;
                        return Ok(());
                    }

                    // Safe command - execute immediately
                    self.execute_validated_command(terminal, &command).await?;
                }
                Err(e) => {
                    // Validation failed - show error
                    let cmd_output = CommandOutput::new(
                        command,
                        String::new(),
                        format!("Command rejected by security validator: {}", e),
                        1,
                    );
                    self.output.set_output(cmd_output);
                    self.state = AppState::ShowingOutput;
                }
            }
        }
        Ok(())
    }

    async fn execute_validated_command<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        command: &str,
    ) -> io::Result<()> {
        self.state = AppState::Executing;
        terminal.draw(|f| self.render(f))?; // Show "Executing..." message

        // Strip "git " prefix if present - executor adds it
        let command_for_executor = command.strip_prefix("git ").unwrap_or(command);

        // Execute via git executor
        let result = self.repo.executor().execute(command_for_executor);

        match result {
            Ok(output) => {
                let cmd_output = CommandOutput::new(
                    command.to_string(),
                    output.stdout,
                    output.stderr,
                    output.exit_code,
                );
                self.output.set_output(cmd_output);

                // Refresh repo state after command
                let _ = self.refresh_repo_state();
            }
            Err(e) => {
                // Translate error to user-friendly message
                let friendly = ErrorTranslator::translate(&e);
                let error_msg = if let Some(ref suggestion) = friendly.suggestion {
                    format!("{}\n\n💡 {}", friendly.simple_message, suggestion)
                } else {
                    friendly.simple_message.clone()
                };

                let cmd_output = CommandOutput::new(
                    command.to_string(),
                    String::new(),
                    error_msg,
                    1,
                );
                self.output.set_output(cmd_output);
            }
        }

        self.state = AppState::ShowingOutput;
        Ok(())
    }

    fn handle_output_state(&mut self, _key: KeyEvent) {
        // Any key returns to input
        self.output.clear();
        self.preview = None;
        self.pending_query = None;
        self.state = AppState::Input;
    }

    /// Refresh repository state
    pub fn refresh_repo_state(&mut self) -> Result<()> {
        match self.repo.state() {
            Ok(state) => {
                self.repo_state = state;
                if self.translator.is_some() {
                    self.mode = AppMode::Normal;
                }
                Ok(())
            }
            Err(e) => {
                self.mode = AppMode::Offline;
                Err(e)
            }
        }
    }

    /// Check if the app should quit
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Handle key input in dangerous operation confirmation state
    async fn handle_confirm_dangerous_state<B: Backend>(
        &mut self,
        key: KeyEvent,
        terminal: &mut Terminal<B>,
    ) -> io::Result<()> {
        match key.code {
            KeyCode::Char(c) => {
                self.confirmation_input.push(c);
            }
            KeyCode::Backspace => {
                self.confirmation_input.pop();
            }
            KeyCode::Enter => {
                if self.confirmation_input == "CONFIRM" {
                    // User confirmed - execute the command
                    if let Some(ref preview) = self.preview {
                        let command = preview.get_command().to_string();
                        self.execute_validated_command(terminal, &command).await?;
                    }
                    self.confirmation_input.clear();
                    self.dangerous_op_type = None;
                } else {
                    // Invalid confirmation - show error
                    self.error_message = Some("Must type CONFIRM exactly".to_string());
                }
            }
            KeyCode::Esc => {
                // Cancel dangerous operation
                self.confirmation_input.clear();
                self.dangerous_op_type = None;
                self.preview = None;
                self.state = AppState::Input;
            }
            _ => {}
        }
        Ok(())
    }

    /// Render dangerous operation confirmation dialog
    fn render_dangerous_confirmation(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::text::{Line, Span};
        use ratatui::widgets::Wrap;

        let danger_desc = match self.dangerous_op_type {
            Some(crate::security::DangerousOp::ForcePush) => {
                "⚠️  FORCE PUSH - This will overwrite remote history!"
            }
            Some(crate::security::DangerousOp::HardReset) => {
                "⚠️  HARD RESET - This will permanently discard changes!"
            }
            Some(crate::security::DangerousOp::Clean) => {
                "⚠️  CLEAN - This will permanently delete untracked files!"
            }
            Some(crate::security::DangerousOp::FilterBranch) => {
                "⚠️  FILTER-BRANCH - This will rewrite git history!"
            }
            None => "⚠️  DANGEROUS OPERATION",
        };

        let command = self
            .preview
            .as_ref()
            .map(|p| p.get_command())
            .unwrap_or("");

        let mut lines = vec![
            Line::from(vec![Span::styled(
                danger_desc,
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Command: ", Style::default().fg(Color::Yellow)),
                Span::styled(command, Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "Type CONFIRM to execute: ",
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    &self.confirmation_input,
                    Style::default().fg(Color::White),
                ),
                Span::styled("█", Style::default().fg(Color::Yellow)),
            ]),
        ];

        // Show hint if input is wrong
        if !self.confirmation_input.is_empty() && !self.confirmation_input.starts_with("CONFIRM") {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "(Must match exactly: CONFIRM)",
                Style::default().fg(Color::Red),
            )]));
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red))
            .title("⚠️  DANGEROUS OPERATION - CONFIRM");

        let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        // This test requires a real git repo
        if let Ok(repo) = Repository::discover() {
            let app = App::new(repo);
            assert!(app.is_ok());
        }
    }

    #[test]
    fn test_offline_mode_without_api_key() {
        // Ensure API key is not set
        unsafe {
            env::remove_var("ANTHROPIC_API_KEY");
        }

        if let Ok(repo) = Repository::discover() {
            let app = App::new(repo).unwrap();
            assert_eq!(app.mode, AppMode::Offline);
            assert!(app.translator.is_none());
        }
    }
}
