pub mod commits;
pub mod diff;
pub mod files;

use crate::app::{App, Panel};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::Paragraph,
};

/// Helper to color the borders based on focus
pub fn border_style(focused: bool) -> Style {
    if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    }
}

pub fn render(frame: &mut Frame, app: &mut App) {
    // ── Outer layout: commit list / bottom section / status bar ──
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(70),
            Constraint::Length(1),
        ])
        .split(frame.area());

    // ── Bottom section: file tree | diff viewer ──────────────────
    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(outer[1]);

    // ── Delegate rendering to submodules ─────────────────────────
    commits::render(frame, app, outer[0]);
    files::render(frame, app, bottom[0]);
    diff::render(frame, app, bottom[1]);

    // ── Status bar ───────────────────────────────────────────────
    let hint = match app.active_panel {
        Panel::CommitList => " j/k: navigate   Enter: open commit   Tab: focus next   q: quit",
        Panel::FileTree => " j/k: switch file   Tab: view diff   Esc: commits   q: quit",
        Panel::Diff => " j/k: scroll   Esc: files   q: quit",
    };
    frame.render_widget(Paragraph::new(hint), outer[2]);
}
