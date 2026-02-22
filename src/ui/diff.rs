use crate::app::{App, DiffLine, DiffLineKind, Panel, SbsRow, build_side_by_side};
use crate::ui::border_style;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let diff_lines = current_diff_lines(app);
    let sbs_rows = build_side_by_side(diff_lines);

    let (left_lines, right_lines) = render_sbs_rows(&sbs_rows);
    let diff_title = app
        .file_list_state
        .selected()
        .and_then(|i| app.changed_files.get(i))
        .map(|f| format!(" {} ", f.path))
        .unwrap_or_else(|| " Diff ".to_string());

    let is_focused = app.active_panel == Panel::Diff;

    // 1. Create the outer block with borders and title
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .title(diff_title)
        .border_style(border_style(is_focused));

    // Calculate the usable space inside the borders
    let inner_area = outer_block.inner(area);

    // Render the outer block first
    frame.render_widget(outer_block, area);

    // 2. Split the inside area perfectly in half
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner_area);

    // 3. Create two paragraphs and sync their vertical scroll!
    let left_widget = Paragraph::new(left_lines).scroll((app.diff_scroll, 0));
    let right_widget = Paragraph::new(right_lines).scroll((app.diff_scroll, 0));

    // Render them side-by-side
    frame.render_widget(left_widget, chunks[0]);
    frame.render_widget(right_widget, chunks[1]);
}

// ── Helpers migrated from original code ─────────────────────────────────

fn current_diff_lines(app: &App) -> &[DiffLine] {
    match app.file_list_state.selected() {
        Some(idx) => app
            .changed_files
            .get(idx)
            .map(|f| f.diff_lines.as_slice())
            .unwrap_or(&[]),
        None => &[],
    }
}

/// Takes our 2D data model and returns two parallel columns of UI Lines
fn render_sbs_rows(rows: &[SbsRow]) -> (Vec<Line<'_>>, Vec<Line<'_>>) {
    let mut left_lines = Vec::new();
    let mut right_lines = Vec::new();

    for row in rows {
        left_lines.push(render_half(&row.left));
        right_lines.push(render_half(&row.right));
    }
    (left_lines, right_lines)
}

/// Formats a single DiffLine, or prints a `~` if the side is empty
fn render_half(line: &Option<DiffLine>) -> Line<'_> {
    match line {
        Some(dl) => {
            let style = match dl.kind {
                DiffLineKind::Added => Style::default().fg(Color::Green),
                DiffLineKind::Removed => Style::default().fg(Color::Red),
                DiffLineKind::Header => Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
                DiffLineKind::Normal => Style::default().fg(Color::Gray),
            };
            Line::from(Span::styled(dl.content.as_str(), style))
        }
        None => {
            // A subtle tilde indicates missing lines (like Vim does!)
            Line::from(Span::styled("~", Style::default().fg(Color::DarkGray)))
        }
    }
}
