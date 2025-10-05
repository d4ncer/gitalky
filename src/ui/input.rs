use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Widget},
};

/// Input mode determines the prompt text
#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Online,  // LLM available
    Offline, // Direct git commands only
}

/// Text input widget for natural language queries or git commands
pub struct InputWidget {
    input: String,
    cursor_position: usize,
    mode: InputMode,
    active: bool,
}

impl InputWidget {
    pub fn new(mode: InputMode) -> Self {
        Self {
            input: String::new(),
            cursor_position: 0,
            mode,
            active: false,
        }
    }

    /// Set whether the input widget is active (focused)
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    /// Set the input mode
    pub fn set_mode(&mut self, mode: InputMode) {
        self.mode = mode;
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(c) => {
                // Check for Ctrl+C (don't insert)
                if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'c' {
                    return false;
                }

                self.input.insert(self.cursor_position, c);
                self.cursor_position += 1;
                true
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    self.input.remove(self.cursor_position);
                }
                true
            }
            KeyCode::Delete => {
                if self.cursor_position < self.input.len() {
                    self.input.remove(self.cursor_position);
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
                if self.cursor_position < self.input.len() {
                    self.cursor_position += 1;
                }
                true
            }
            KeyCode::Home => {
                self.cursor_position = 0;
                true
            }
            KeyCode::End => {
                self.cursor_position = self.input.len();
                true
            }
            _ => false,
        }
    }

    /// Take the current input and clear the widget
    pub fn take_input(&mut self) -> String {
        let input = self.input.clone();
        self.input.clear();
        self.cursor_position = 0;
        input
    }

    /// Get the current input (without clearing)
    pub fn get_input(&self) -> &str {
        &self.input
    }

    /// Clear the input
    pub fn clear(&mut self) {
        self.input.clear();
        self.cursor_position = 0;
    }

    /// Get prompt text based on mode
    fn get_prompt(&self) -> &str {
        match self.mode {
            InputMode::Online => "Natural language or git command:",
            InputMode::Offline => "Enter git command:",
        }
    }
}

impl Widget for &InputWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let prompt = self.get_prompt();

        // Create display text with cursor
        let display_text = if self.active {
            // Show cursor at position
            let before = &self.input[..self.cursor_position];
            let after = &self.input[self.cursor_position..];
            format!("{} {}▊{}", prompt, before, after)
        } else {
            format!("{} {}", prompt, self.input)
        };

        let style = if self.active {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(if self.active {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::DarkGray)
            });

        let paragraph = Paragraph::new(display_text)
            .style(style)
            .block(block);

        paragraph.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    #[test]
    fn test_input_widget_creation() {
        let widget = InputWidget::new(InputMode::Online);
        assert_eq!(widget.get_input(), "");
        assert_eq!(widget.cursor_position, 0);
    }

    #[test]
    fn test_input_char() {
        let mut widget = InputWidget::new(InputMode::Online);
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);

        assert!(widget.handle_key(key));
        assert_eq!(widget.get_input(), "a");
        assert_eq!(widget.cursor_position, 1);
    }

    #[test]
    fn test_input_backspace() {
        let mut widget = InputWidget::new(InputMode::Online);
        widget.handle_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
        widget.handle_key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE));

        assert_eq!(widget.get_input(), "ab");

        widget.handle_key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        assert_eq!(widget.get_input(), "a");
        assert_eq!(widget.cursor_position, 1);
    }

    #[test]
    fn test_input_cursor_movement() {
        let mut widget = InputWidget::new(InputMode::Online);
        widget.handle_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
        widget.handle_key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE));
        widget.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE));

        assert_eq!(widget.cursor_position, 3);

        widget.handle_key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
        assert_eq!(widget.cursor_position, 2);

        widget.handle_key(KeyEvent::new(KeyCode::Home, KeyModifiers::NONE));
        assert_eq!(widget.cursor_position, 0);

        widget.handle_key(KeyEvent::new(KeyCode::End, KeyModifiers::NONE));
        assert_eq!(widget.cursor_position, 3);
    }

    #[test]
    fn test_take_input() {
        let mut widget = InputWidget::new(InputMode::Online);
        widget.handle_key(KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE));
        widget.handle_key(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE));
        widget.handle_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE));
        widget.handle_key(KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE));

        let input = widget.take_input();
        assert_eq!(input, "test");
        assert_eq!(widget.get_input(), "");
        assert_eq!(widget.cursor_position, 0);
    }

    #[test]
    fn test_prompt_changes_with_mode() {
        let online = InputWidget::new(InputMode::Online);
        assert_eq!(online.get_prompt(), "Natural language or git command:");

        let offline = InputWidget::new(InputMode::Offline);
        assert_eq!(offline.get_prompt(), "Enter git command:");
    }
}
