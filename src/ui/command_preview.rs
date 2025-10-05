use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

/// Command preview widget for reviewing and editing proposed commands
pub struct CommandPreview {
    command: String,
    explanation: Option<String>,
    edit_mode: bool,
    cursor_position: usize,
}

impl CommandPreview {
    pub fn new(command: String, explanation: Option<String>) -> Self {
        let cursor_position = command.len();
        Self {
            command,
            explanation,
            edit_mode: false,
            cursor_position,
        }
    }

    /// Enter edit mode for modifying the command
    pub fn enter_edit_mode(&mut self) {
        self.edit_mode = true;
        self.cursor_position = self.command.len();
    }

    /// Exit edit mode
    pub fn exit_edit_mode(&mut self) {
        self.edit_mode = false;
    }

    /// Check if in edit mode
    pub fn is_edit_mode(&self) -> bool {
        self.edit_mode
    }

    /// Get the current command
    pub fn get_command(&self) -> &str {
        &self.command
    }

    /// Handle keyboard input in edit mode
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if !self.edit_mode {
            return false;
        }

        match key.code {
            KeyCode::Char(c) => {
                // Check for Ctrl+C (don't insert)
                if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'c' {
                    return false;
                }

                self.command.insert(self.cursor_position, c);
                self.cursor_position += 1;
                true
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    self.command.remove(self.cursor_position);
                }
                true
            }
            KeyCode::Delete => {
                if self.cursor_position < self.command.len() {
                    self.command.remove(self.cursor_position);
                }
                true
            }
            KeyCode::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
                true
            }
            KeyCode::Right => {
                if self.cursor_position < self.command.len() {
                    self.cursor_position += 1;
                }
                true
            }
            KeyCode::Home => {
                self.cursor_position = 0;
                true
            }
            KeyCode::End => {
                self.cursor_position = self.command.len();
                true
            }
            _ => false,
        }
    }
}

impl Widget for &CommandPreview {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut lines = Vec::new();

        // Title line
        if self.edit_mode {
            lines.push(Line::from(vec![
                Span::styled("Command (editing)", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled("Proposed Command", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]));
        }

        lines.push(Line::from(""));

        // Command line with cursor if in edit mode
        if self.edit_mode {
            let before = &self.command[..self.cursor_position];
            let after = &self.command[self.cursor_position..];
            lines.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(before, Style::default().fg(Color::Green)),
                Span::styled("▊", Style::default().fg(Color::Yellow)),
                Span::styled(after, Style::default().fg(Color::Green)),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(&self.command, Style::default().fg(Color::Green)),
            ]));
        }

        // Explanation if present
        if let Some(ref explanation) = self.explanation {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Explanation: ", Style::default().fg(Color::DarkGray)),
                Span::styled(explanation, Style::default().fg(Color::Gray)),
            ]));
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(if self.edit_mode {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Cyan)
            })
            .title("Command Preview");

        let paragraph = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false });

        paragraph.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_preview_creation() {
        let preview = CommandPreview::new("git status".to_string(), None);
        assert_eq!(preview.get_command(), "git status");
        assert!(!preview.is_edit_mode());
    }

    #[test]
    fn test_command_preview_with_explanation() {
        let preview = CommandPreview::new(
            "git status".to_string(),
            Some("Shows working tree status".to_string()),
        );
        assert_eq!(preview.get_command(), "git status");
        assert!(preview.explanation.is_some());
    }

    #[test]
    fn test_edit_mode() {
        let mut preview = CommandPreview::new("git status".to_string(), None);
        assert!(!preview.is_edit_mode());

        preview.enter_edit_mode();
        assert!(preview.is_edit_mode());

        preview.exit_edit_mode();
        assert!(!preview.is_edit_mode());
    }

    #[test]
    fn test_edit_command() {
        let mut preview = CommandPreview::new("git status".to_string(), None);
        preview.enter_edit_mode();

        // Add text
        let key = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE);
        assert!(preview.handle_key(key));

        let key = KeyEvent::new(KeyCode::Char('-'), KeyModifiers::NONE);
        assert!(preview.handle_key(key));

        let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE);
        assert!(preview.handle_key(key));

        assert_eq!(preview.get_command(), "git status -s");
    }

    #[test]
    fn test_edit_backspace() {
        let mut preview = CommandPreview::new("git status".to_string(), None);
        preview.enter_edit_mode();

        // Backspace at end
        let key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        assert!(preview.handle_key(key));

        assert_eq!(preview.get_command(), "git statu");
    }

    #[test]
    fn test_no_edit_when_not_in_edit_mode() {
        let mut preview = CommandPreview::new("git status".to_string(), None);

        let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        assert!(!preview.handle_key(key));

        assert_eq!(preview.get_command(), "git status");
    }
}
