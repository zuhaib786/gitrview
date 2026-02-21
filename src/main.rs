use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use git2::Repository;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::io::stdout;

// This struct is the entire memory of our app
struct App {
    commits: Vec<String>,
    list_state: ListState, // ratatui tracks which item is highlighted
}

impl App {
    fn new(commits: Vec<String>) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0)); // start with first item selected
        Self {
            commits,
            list_state,
        }
    }

    fn move_down(&mut self) {
        let current = self.list_state.selected().unwrap_or(0);
        let next = (current + 1).min(self.commits.len().saturating_sub(1));
        self.list_state.select(Some(next));
    }

    fn move_up(&mut self) {
        let current = self.list_state.selected().unwrap_or(0);
        let prev = current.saturating_sub(1);
        self.list_state.select(Some(prev));
    }
}

fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let commits = load_commits()?;
    let mut app = App::new(commits);

    loop {
        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(1)])
                .split(frame.area());

            let items: Vec<ListItem> = app
                .commits
                .iter()
                .map(|c| ListItem::new(c.as_str()))
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Commits"))
                // This style applies to the selected item
                .highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("▶ ");

            // Notice: render_stateful_widget instead of render_widget
            frame.render_stateful_widget(list, chunks[0], &mut app.list_state);

            let status = Paragraph::new(" j/k: navigate  q: quit");
            frame.render_widget(status, chunks[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('j') | KeyCode::Down => app.move_down(),
                KeyCode::Char('k') | KeyCode::Up => app.move_up(),
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn load_commits() -> anyhow::Result<Vec<String>> {
    let repo = Repository::open(".")?;
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    let commits = revwalk
        .take(20)
        .filter_map(|id| {
            let id = id.ok()?;
            let commit = repo.find_commit(id).ok()?;
            let message = commit.summary()?.to_string();
            let short_id = &id.to_string()[..7];
            Some(format!("{} {}", short_id, message))
        })
        .collect();

    Ok(commits)
}
