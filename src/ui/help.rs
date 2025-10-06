use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub struct HelpScreen {
    pub visible: bool,
}

impl HelpScreen {
    pub fn new() -> Self {
        HelpScreen { visible: false }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" Gitalky Help ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(12), // Keyboard shortcuts
                Constraint::Length(1),  // Separator
                Constraint::Length(8),  // Example queries
                Constraint::Length(1),  // Separator
                Constraint::Min(0),     // Config info
            ])
            .split(inner);

        // Keyboard shortcuts
        let shortcuts = vec![
            Line::from(vec![
                Span::styled("Keyboard Shortcuts:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  q          ", Style::default().fg(Color::Cyan)),
                Span::raw("Quit application"),
            ]),
            Line::from(vec![
                Span::styled("  ?          ", Style::default().fg(Color::Cyan)),
                Span::raw("Show/hide this help"),
            ]),
            Line::from(vec![
                Span::styled("  Esc        ", Style::default().fg(Color::Cyan)),
                Span::raw("Cancel current operation"),
            ]),
            Line::from(vec![
                Span::styled("  Enter      ", Style::default().fg(Color::Cyan)),
                Span::raw("Submit query / Execute command"),
            ]),
            Line::from(vec![
                Span::styled("  e          ", Style::default().fg(Color::Cyan)),
                Span::raw("Edit proposed command"),
            ]),
            Line::from(vec![
                Span::styled("  t          ", Style::default().fg(Color::Cyan)),
                Span::raw("Toggle raw/simplified error display"),
            ]),
            Line::from(vec![
                Span::styled("  r          ", Style::default().fg(Color::Cyan)),
                Span::raw("Retry LLM connection (when offline)"),
            ]),
        ];

        let shortcuts_widget = Paragraph::new(shortcuts)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false });
        frame.render_widget(shortcuts_widget, chunks[0]);

        // Separator
        let sep1 = Paragraph::new(Line::from("─".repeat(inner.width as usize)))
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(sep1, chunks[1]);

        // Example queries
        let examples = vec![
            Line::from(vec![
                Span::styled("Example Queries:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  • ", Style::default().fg(Color::Green)),
                Span::raw("\"show me what changed\""),
            ]),
            Line::from(vec![
                Span::styled("  • ", Style::default().fg(Color::Green)),
                Span::raw("\"commit these changes with message 'fix bug'\""),
            ]),
            Line::from(vec![
                Span::styled("  • ", Style::default().fg(Color::Green)),
                Span::raw("\"create a new branch called feature-x from main\""),
            ]),
            Line::from(vec![
                Span::styled("  • ", Style::default().fg(Color::Green)),
                Span::raw("\"show me the last 10 commits\""),
            ]),
        ];

        let examples_widget = Paragraph::new(examples)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false });
        frame.render_widget(examples_widget, chunks[2]);

        // Separator
        let sep2 = Paragraph::new(Line::from("─".repeat(inner.width as usize)))
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(sep2, chunks[3]);

        // Config info
        let config_info = vec![
            Line::from(vec![
                Span::styled("Configuration:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Config file:  ", Style::default().fg(Color::Cyan)),
                Span::raw("~/.config/gitalky/config.toml"),
            ]),
            Line::from(vec![
                Span::styled("  Audit log:    ", Style::default().fg(Color::Cyan)),
                Span::raw("~/.config/gitalky/history.log"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Press '?' or Esc to close this help screen",
                    Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
            ]),
        ];

        let config_widget = Paragraph::new(config_info)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false });
        frame.render_widget(config_widget, chunks[4]);
    }
}

impl Default for HelpScreen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_screen_creation() {
        let help = HelpScreen::new();
        assert!(!help.visible);
    }

    #[test]
    fn test_help_screen_toggle() {
        let mut help = HelpScreen::new();
        assert!(!help.visible);

        help.toggle();
        assert!(help.visible);

        help.toggle();
        assert!(!help.visible);
    }

    #[test]
    fn test_help_screen_hide() {
        let mut help = HelpScreen::new();
        help.visible = true;

        help.hide();
        assert!(!help.visible);
    }
}
