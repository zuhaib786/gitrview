use crate::app::{App, Panel};
use crate::ui::border_style;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let commit_items: Vec<ListItem> = app
        .commits
        .iter()
        .map(|c| {
            let spans = vec![
                Span::styled(c.short_id.clone(), Style::default().fg(Color::Yellow)),
                Span::raw(&c.summary),
            ];
            ListItem::new(Line::from(spans))
        })
        .collect();

    let is_focused = app.active_panel == Panel::CommitList;

    let commit_list = List::new(commit_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Commits ")
                .border_style(border_style(is_focused)),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(commit_list, area, &mut app.commit_list_state);
}
