use crate::error::Result;
use crate::git::{Repository, RepositoryState};
use crate::ui::repo_panel::RepositoryPanel;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    Offline,
}

/// Main application state
pub struct App {
    repo: Repository,
    repo_state: RepositoryState,
    should_quit: bool,
    mode: AppMode,
}

impl App {
    /// Create a new App instance with the given repository
    pub fn new(repo: Repository) -> Result<Self> {
        let repo_state = repo.state()?;

        Ok(Self {
            repo,
            repo_state,
            should_quit: false,
            mode: AppMode::Normal,
        })
    }

    /// Run the application event loop
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        loop {
            terminal.draw(|f| self.render(f))?;

            // Poll for events with 100ms timeout for refresh
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key_event(key);
                }
            } else {
                // Refresh repository state on timeout
                if let Err(e) = self.refresh_repo_state() {
                    // If refresh fails, switch to offline mode
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
        let size = frame.area();

        // Create layout: title bar + content
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
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

        // Status bar (help text)
        let status_text = "Press 'q' to quit";
        let status_para = Paragraph::new(status_text);
        let status_area = ratatui::layout::Rect {
            x: 0,
            y: size.height.saturating_sub(1),
            width: size.width,
            height: 1,
        };
        frame.render_widget(status_para, status_area);
    }

    /// Handle keyboard events
    fn handle_key_event(&mut self, key: KeyEvent) {
        // Only handle key press events (not release or repeat)
        if key.kind != KeyEventKind::Press {
            return;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    /// Refresh repository state
    pub fn refresh_repo_state(&mut self) -> Result<()> {
        match self.repo.state() {
            Ok(state) => {
                self.repo_state = state;
                self.mode = AppMode::Normal;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        // This test requires a real git repo, so we'll use the current repo
        if let Ok(repo) = Repository::discover() {
            let app = App::new(repo);
            assert!(app.is_ok());
        }
    }

    #[test]
    fn test_quit_on_q_key() {
        if let Ok(repo) = Repository::discover() {
            let mut app = App::new(repo).unwrap();
            assert!(!app.should_quit());

            let key_event = KeyEvent::new(KeyCode::Char('q'), event::KeyModifiers::NONE);
            app.handle_key_event(key_event);

            assert!(app.should_quit());
        }
    }

    #[test]
    fn test_offline_mode_on_error() {
        if let Ok(repo) = Repository::discover() {
            let mut app = App::new(repo).unwrap();
            assert_eq!(app.mode, AppMode::Normal);

            // Mode should remain normal on successful refresh
            let _ = app.refresh_repo_state();
            assert_eq!(app.mode, AppMode::Normal);
        }
    }
}
