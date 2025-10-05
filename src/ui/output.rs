use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

/// Command execution result
#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub command: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

impl CommandOutput {
    pub fn new(command: String, stdout: String, stderr: String, exit_code: i32) -> Self {
        Self {
            command,
            stdout,
            stderr,
            exit_code,
        }
    }

    pub fn is_success(&self) -> bool {
        self.exit_code == 0
    }
}

/// Output display widget for showing command execution results
pub struct OutputDisplay {
    output: Option<CommandOutput>,
    scroll: usize,
}

impl OutputDisplay {
    pub fn new() -> Self {
        Self {
            output: None,
            scroll: 0,
        }
    }

    /// Set the output to display
    pub fn set_output(&mut self, output: CommandOutput) {
        self.output = Some(output);
        self.scroll = 0;
    }

    /// Clear the output
    pub fn clear(&mut self) {
        self.output = None;
        self.scroll = 0;
    }

    /// Scroll up
    pub fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
        }
    }

    /// Scroll down
    pub fn scroll_down(&mut self) {
        self.scroll += 1;
    }
}

impl Default for OutputDisplay {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for &OutputDisplay {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if let Some(ref output) = self.output {
            let mut lines = Vec::new();

            // Header with status
            let status_style = if output.is_success() {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            };

            let status_text = if output.is_success() {
                "✓ Success".to_string()
            } else {
                format!("✗ Failed (exit code: {})", output.exit_code)
            };

            lines.push(Line::from(vec![
                Span::styled("Command: ", Style::default().fg(Color::Cyan)),
                Span::styled(&output.command, Style::default().fg(Color::White)),
            ]));

            lines.push(Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::Cyan)),
                Span::styled(status_text, status_style),
            ]));

            lines.push(Line::from(""));

            // Stdout
            if !output.stdout.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("Output:", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                ]));

                for line in output.stdout.lines() {
                    lines.push(Line::from(vec![
                        Span::styled(line, Style::default().fg(Color::White)),
                    ]));
                }

                lines.push(Line::from(""));
            }

            // Stderr
            if !output.stderr.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("Errors:", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                ]));

                for line in output.stderr.lines() {
                    lines.push(Line::from(vec![
                        Span::styled(line, Style::default().fg(Color::Red)),
                    ]));
                }
            }

            // Apply scrolling by skipping lines
            let visible_lines: Vec<_> = lines.into_iter().skip(self.scroll).collect();

            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(if output.is_success() {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Red)
                })
                .title("Command Output");

            let paragraph = Paragraph::new(visible_lines)
                .block(block)
                .wrap(Wrap { trim: false });

            paragraph.render(area, buf);
        } else {
            // No output to display
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title("Command Output");

            let paragraph = Paragraph::new("No command executed yet")
                .style(Style::default().fg(Color::DarkGray))
                .block(block);

            paragraph.render(area, buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_display_creation() {
        let display = OutputDisplay::new();
        assert!(display.output.is_none());
        assert_eq!(display.scroll, 0);
    }

    #[test]
    fn test_command_output_success() {
        let output = CommandOutput::new(
            "git status".to_string(),
            "On branch main".to_string(),
            String::new(),
            0,
        );

        assert!(output.is_success());
        assert_eq!(output.command, "git status");
        assert_eq!(output.stdout, "On branch main");
    }

    #[test]
    fn test_command_output_failure() {
        let output = CommandOutput::new(
            "git invalid".to_string(),
            String::new(),
            "invalid command".to_string(),
            1,
        );

        assert!(!output.is_success());
        assert_eq!(output.exit_code, 1);
        assert_eq!(output.stderr, "invalid command");
    }

    #[test]
    fn test_set_and_clear_output() {
        let mut display = OutputDisplay::new();
        let output = CommandOutput::new(
            "git status".to_string(),
            "output".to_string(),
            String::new(),
            0,
        );

        display.set_output(output);
        assert!(display.output.is_some());

        display.clear();
        assert!(display.output.is_none());
        assert_eq!(display.scroll, 0);
    }

    #[test]
    fn test_scroll() {
        let mut display = OutputDisplay::new();
        assert_eq!(display.scroll, 0);

        display.scroll_down();
        assert_eq!(display.scroll, 1);

        display.scroll_down();
        assert_eq!(display.scroll, 2);

        display.scroll_up();
        assert_eq!(display.scroll, 1);

        display.scroll_up();
        assert_eq!(display.scroll, 0);

        // Can't scroll below 0
        display.scroll_up();
        assert_eq!(display.scroll, 0);
    }
}
