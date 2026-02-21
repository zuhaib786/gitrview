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
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::io::stdout;

fn main() -> Result<()> {
    // --- Setup terminal ---
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // --- Load commits from the current directory's git repo ---
    let commits = load_commits()?;

    // --- Main loop ---
    loop {
        terminal.draw(|frame| {
            // Split the screen into two vertical sections
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),    // commit list takes remaining space
                    Constraint::Length(1), // status bar is exactly 1 line tall
                ])
                .split(frame.area());

            // Build a list widget from our commits
            let items: Vec<ListItem> = commits.iter().map(|c| ListItem::new(c.as_str())).collect();

            let list =
                List::new(items).block(Block::default().borders(Borders::ALL).title("Commits"));

            frame.render_widget(list, chunks[0]);

            // Status bar
            let status = Paragraph::new(" q: quit");
            frame.render_widget(status, chunks[1]);
        })?;

        // Handle input
        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') {
                break;
            }
        }
    }

    // --- Teardown terminal (IMPORTANT: always restore terminal state) ---
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn load_commits() -> Result<Vec<String>> {
    let repo = Repository::open(".")?;
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    let commits = revwalk
        .take(20) // only load last 20 for now
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
