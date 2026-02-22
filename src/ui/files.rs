use crate::app::{App, Panel};
use crate::ui::border_style;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
};

pub fn render(frame: &mut Frame, app: &mut App, area: Rect) {
    let file_items: Vec<ListItem> = app
        .changed_files
        .iter()
        .map(|f| ListItem::new(f.path.as_str()))
        .collect();

    let is_focused = app.active_panel == Panel::FileTree;

    let file_list = List::new(file_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Files ")
                .border_style(border_style(is_focused)),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(file_list, area, &mut app.file_list_state);
}
