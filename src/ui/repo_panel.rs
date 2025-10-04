use crate::git::{FileStatus, RepositoryState};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

/// Repository state display panel
pub struct RepositoryPanel<'a> {
    state: &'a RepositoryState,
}

impl<'a> RepositoryPanel<'a> {
    pub fn new(state: &'a RepositoryState) -> Self {
        Self { state }
    }

    /// Build the content lines for the repository panel
    fn build_content(&self) -> Vec<Line<'a>> {
        let mut lines = Vec::new();

        // Repository State header
        lines.push(Line::from(Span::styled(
            "Repository State",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from("─".repeat(60)));
        lines.push(Line::from(""));

        // Head section
        self.add_head_section(&mut lines);
        lines.push(Line::from(""));

        // Untracked files
        if !self.state.untracked_files.is_empty() {
            self.add_untracked_section(&mut lines);
            lines.push(Line::from(""));
        }

        // Unstaged changes
        if !self.state.unstaged_files.is_empty() {
            self.add_unstaged_section(&mut lines);
            lines.push(Line::from(""));
        }

        // Staged changes
        if !self.state.staged_files.is_empty() {
            self.add_staged_section(&mut lines);
            lines.push(Line::from(""));
        }

        // Stashes (only show if stashes exist)
        if !self.state.stashes.is_empty() {
            self.add_stash_section(&mut lines);
            lines.push(Line::from(""));
        }

        // Recent commits
        self.add_commits_section(&mut lines);

        lines
    }

    fn add_head_section(&self, lines: &mut Vec<Line<'a>>) {
        let mut head_spans = vec![];

        if let Some(ref branch) = self.state.current_branch {
            head_spans.push(Span::styled(
                format!("Head:     {}", branch),
                Style::default().fg(Color::Yellow),
            ));

            // Add upstream tracking info if available
            if let Some(ref upstream) = self.state.upstream {
                let mut tracking_parts = vec![];

                if upstream.ahead > 0 {
                    tracking_parts.push(format!("↑{}", upstream.ahead));
                }
                if upstream.behind > 0 {
                    tracking_parts.push(format!("↓{}", upstream.behind));
                }

                if !tracking_parts.is_empty() {
                    head_spans.push(Span::raw(" "));
                    head_spans.push(Span::styled(
                        tracking_parts.join(" "),
                        Style::default().fg(Color::Cyan),
                    ));
                }

                head_spans.push(Span::raw("  "));
                head_spans.push(Span::styled(
                    format!("({})", upstream.remote_branch),
                    Style::default().fg(Color::DarkGray),
                ));
            }
        } else {
            head_spans.push(Span::styled(
                "Head:     (detached HEAD)".to_string(),
                Style::default().fg(Color::Yellow),
            ));
        }

        lines.push(Line::from(head_spans));
    }

    fn add_untracked_section(&self, lines: &mut Vec<Line<'a>>) {
        let count = self.state.untracked_files.len();
        lines.push(Line::from(Span::styled(
            format!("Untracked files ({})", count),
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        )));

        for file in self.state.untracked_files.iter().take(10) {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("untracked:  ", Style::default().fg(Color::Red)),
                Span::raw(&file.path),
            ]));
        }

        if count > 10 {
            lines.push(Line::from(Span::styled(
                format!("  ... and {} more", count - 10),
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    fn add_unstaged_section(&self, lines: &mut Vec<Line<'a>>) {
        let count = self.state.unstaged_files.len();
        lines.push(Line::from(Span::styled(
            format!("Unstaged changes ({})", count),
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        )));

        for file in self.state.unstaged_files.iter().take(10) {
            let (status_text, color) = match file.status {
                FileStatus::Modified => ("modified:  ", Color::Yellow),
                FileStatus::Deleted => ("deleted:   ", Color::Red),
                FileStatus::Added => ("new file:  ", Color::Green),
                _ => ("unknown:   ", Color::White),
            };

            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(status_text, Style::default().fg(color)),
                Span::raw(&file.path),
            ]));
        }

        if count > 10 {
            lines.push(Line::from(Span::styled(
                format!("  ... and {} more", count - 10),
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    fn add_staged_section(&self, lines: &mut Vec<Line<'a>>) {
        let count = self.state.staged_files.len();
        lines.push(Line::from(Span::styled(
            format!("Staged changes ({})", count),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )));

        for file in self.state.staged_files.iter().take(10) {
            let (status_text, color) = match file.status {
                FileStatus::Modified => ("modified:  ", Color::Yellow),
                FileStatus::Deleted => ("deleted:   ", Color::Red),
                FileStatus::Added => ("new file:  ", Color::Green),
                _ => ("unknown:   ", Color::White),
            };

            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(status_text, Style::default().fg(color)),
                Span::raw(&file.path),
            ]));
        }

        if count > 10 {
            lines.push(Line::from(Span::styled(
                format!("  ... and {} more", count - 10),
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    fn add_stash_section(&self, lines: &mut Vec<Line<'a>>) {
        let count = self.state.stashes.len();
        lines.push(Line::from(Span::styled(
            format!("Stashes ({})", count),
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )));

        // Show first 5 stashes
        for stash in self.state.stashes.iter().take(5) {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(&stash.index, Style::default().fg(Color::Cyan)),
                Span::raw(": "),
                Span::raw(&stash.message),
            ]));
        }

        if count > 5 {
            lines.push(Line::from(Span::styled(
                format!("  ... and {} more", count - 5),
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    fn add_commits_section(&self, lines: &mut Vec<Line<'a>>) {
        let count = self.state.recent_commits.len();
        let display_count = if count > 0 { count } else { 0 };

        lines.push(Line::from(Span::styled(
            format!("Recent commits ({})", display_count),
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )));

        for commit in self.state.recent_commits.iter().take(5) {
            let short_hash = if commit.hash.len() >= 7 {
                &commit.hash[..7]
            } else {
                &commit.hash
            };

            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(short_hash, Style::default().fg(Color::Yellow)),
                Span::raw(" "),
                Span::raw(&commit.message),
            ]));
        }

        if count > 5 {
            lines.push(Line::from(Span::styled(
                format!("  ... and {} more", count - 5),
                Style::default().fg(Color::DarkGray),
            )));
        }
    }
}

impl<'a> Widget for RepositoryPanel<'a> {
    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let content = self.build_content();
        let paragraph = Paragraph::new(content).block(Block::default().borders(Borders::ALL));
        paragraph.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::{CommitEntry, FileStatus, StashEntry, StatusEntry};

    fn create_test_state() -> RepositoryState {
        RepositoryState {
            current_branch: Some("main".to_string()),
            upstream: None,
            staged_files: vec![StatusEntry {
                status: FileStatus::Added,
                path: "new_file.rs".to_string(),
                staged: true,
                unstaged: false,
            }],
            unstaged_files: vec![StatusEntry {
                status: FileStatus::Modified,
                path: "existing.rs".to_string(),
                staged: false,
                unstaged: true,
            }],
            untracked_files: vec![StatusEntry {
                status: FileStatus::Untracked,
                path: "untracked.txt".to_string(),
                staged: false,
                unstaged: false,
            }],
            recent_commits: vec![
                CommitEntry {
                    hash: "abc123def456".to_string(),
                    message: "Initial commit".to_string(),
                },
                CommitEntry {
                    hash: "def456abc123".to_string(),
                    message: "Second commit".to_string(),
                },
            ],
            stashes: vec![
                StashEntry {
                    index: "stash@{0}".to_string(),
                    message: "WIP on main: work in progress".to_string(),
                },
                StashEntry {
                    index: "stash@{1}".to_string(),
                    message: "WIP on feature: experimental".to_string(),
                },
            ],
            in_merge: false,
            in_rebase: false,
        }
    }

    #[test]
    fn test_panel_creation() {
        let state = create_test_state();
        let panel = RepositoryPanel::new(&state);
        let content = panel.build_content();

        assert!(!content.is_empty());
    }

    #[test]
    fn test_panel_shows_branch() {
        let state = create_test_state();
        let panel = RepositoryPanel::new(&state);
        let content = panel.build_content();

        let has_branch = content.iter().any(|line| {
            line.spans
                .iter()
                .any(|span| span.content.contains("main"))
        });
        assert!(has_branch);
    }

    #[test]
    fn test_panel_shows_stashes() {
        let state = create_test_state();
        let panel = RepositoryPanel::new(&state);
        let content = panel.build_content();

        let has_stash = content.iter().any(|line| {
            line.spans
                .iter()
                .any(|span| span.content.contains("Stashes"))
        });
        assert!(has_stash);

        let has_stash_entry = content.iter().any(|line| {
            line.spans
                .iter()
                .any(|span| span.content.contains("stash@{0}"))
        });
        assert!(has_stash_entry);
    }

    #[test]
    fn test_panel_hides_stashes_when_empty() {
        let mut state = create_test_state();
        state.stashes.clear();

        let panel = RepositoryPanel::new(&state);
        let content = panel.build_content();

        let has_stash_section = content.iter().any(|line| {
            line.spans
                .iter()
                .any(|span| span.content.contains("Stashes"))
        });
        assert!(!has_stash_section);
    }

    #[test]
    fn test_detached_head_display() {
        let mut state = create_test_state();
        state.current_branch = None;

        let panel = RepositoryPanel::new(&state);
        let content = panel.build_content();

        let has_detached = content.iter().any(|line| {
            line.spans
                .iter()
                .any(|span| span.content.contains("detached HEAD"))
        });
        assert!(has_detached);
    }

    #[test]
    fn test_upstream_tracking_display() {
        let mut state = create_test_state();
        state.upstream = Some(crate::git::UpstreamInfo {
            remote_branch: "origin/main".to_string(),
            ahead: 2,
            behind: 1,
        });

        let panel = RepositoryPanel::new(&state);
        let content = panel.build_content();

        // Should show ahead count
        let has_ahead = content.iter().any(|line| {
            line.spans
                .iter()
                .any(|span| span.content.contains("↑2"))
        });
        assert!(has_ahead);

        // Should show behind count
        let has_behind = content.iter().any(|line| {
            line.spans
                .iter()
                .any(|span| span.content.contains("↓1"))
        });
        assert!(has_behind);

        // Should show remote branch
        let has_remote = content.iter().any(|line| {
            line.spans
                .iter()
                .any(|span| span.content.contains("origin/main"))
        });
        assert!(has_remote);
    }

    #[test]
    fn test_no_upstream_display() {
        let state = create_test_state(); // upstream is None

        let panel = RepositoryPanel::new(&state);
        let content = panel.build_content();

        // Should not show any tracking info
        let has_tracking = content.iter().any(|line| {
            line.spans
                .iter()
                .any(|span| span.content.contains("↑") || span.content.contains("↓"))
        });
        assert!(!has_tracking);
    }
}
