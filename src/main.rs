use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use git2::{DiffFormat, Repository};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::io::stdout;

struct App {
    commits: Vec<String>,    // formatted display strings
    commit_ids: Vec<String>, // raw full SHA ids, parallel to commits vec
    list_state: ListState,
    diff_lines: Vec<DiffLine>, // the currently displayed diff
    focused_panel: Panel,
    diff_scroll: u16, // how far we've scrolled in the diff
}

// Which panel keyboard input is currently controlling
#[derive(PartialEq)]
enum Panel {
    CommitList,
    Diff,
}

// A single line of diff output, with its type so we can color it
struct DiffLine {
    content: String,
    kind: DiffLineKind,
}

#[derive(PartialEq)]
enum DiffLineKind {
    Added,
    Removed,
    Header, // the @@ hunk headers
    Normal,
}

impl App {
    fn new(commits: Vec<String>, commit_ids: Vec<String>) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            commits,
            commit_ids,
            list_state,
            diff_lines: vec![],
            focused_panel: Panel::CommitList,
            diff_scroll: 0,
        }
    }

    fn move_down(&mut self) {
        let current = self.list_state.selected().unwrap_or(0);
        let next = (current + 1).min(self.commits.len().saturating_sub(1));
        self.list_state.select(Some(next));
    }

    fn move_up(&mut self) {
        let current = self.list_state.selected().unwrap_or(0);
        self.list_state.select(Some(current.saturating_sub(1)));
    }

    fn scroll_diff_down(&mut self) {
        self.diff_scroll = self.diff_scroll.saturating_add(1);
    }

    fn scroll_diff_up(&mut self) {
        self.diff_scroll = self.diff_scroll.saturating_sub(1);
    }

    fn load_diff(&mut self, repo: &Repository) -> Result<()> {
        let idx = self.list_state.selected().unwrap_or(0);
        let id = git2::Oid::from_str(&self.commit_ids[idx])?;
        let commit = repo.find_commit(id)?;

        // Get the tree for this commit and its parent
        let commit_tree = commit.tree()?;
        let parent_tree = commit
            .parents()
            .next() // first parent (None for initial commit)
            .and_then(|p| p.tree().ok()); // get its tree, or None

        let diff = repo.diff_tree_to_tree(
            parent_tree.as_ref(), // Option<&Tree> — None means "empty tree" (initial commit)
            Some(&commit_tree),
            None,
        )?;

        // Walk every line of the diff and collect it
        let mut lines: Vec<DiffLine> = vec![];
        diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
            let content = std::str::from_utf8(line.content())
                .unwrap_or("")
                .trim_end_matches('\n')
                .to_string();

            let kind = match line.origin() {
                '+' => DiffLineKind::Added,
                '-' => DiffLineKind::Removed,
                'H' => DiffLineKind::Header,
                _ => DiffLineKind::Normal,
            };

            lines.push(DiffLine { content, kind });
            true // returning false would stop iteration
        })?;

        self.diff_lines = lines;
        self.diff_scroll = 0; // reset scroll when loading new diff
        self.focused_panel = Panel::Diff;
        Ok(())
    }
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let repo = Repository::open(".")?;
    let (commits, commit_ids) = load_commits(&repo)?;
    let mut app = App::new(commits, commit_ids);

    loop {
        terminal.draw(|frame| {
            // Split into top (commit list) and bottom (diff)
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Percentage(70),
                    Constraint::Length(1),
                ])
                .split(frame.area());

            // --- Commit list ---
            let items: Vec<ListItem> = app
                .commits
                .iter()
                .map(|c| ListItem::new(c.as_str()))
                .collect();

            let commit_block_style = if app.focused_panel == Panel::CommitList {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            };

            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Commits")
                        .border_style(commit_block_style),
                )
                .highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("▶ ");

            frame.render_stateful_widget(list, chunks[0], &mut app.list_state);

            // --- Diff viewer ---
            let diff_block_style = if app.focused_panel == Panel::Diff {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            };

            // Convert our DiffLines into ratatui colored Lines
            let diff_text: Vec<Line> = app
                .diff_lines
                .iter()
                .map(|dl| {
                    let style = match dl.kind {
                        DiffLineKind::Added => Style::default().fg(Color::Green),
                        DiffLineKind::Removed => Style::default().fg(Color::Red),
                        DiffLineKind::Header => Style::default().fg(Color::Cyan),
                        DiffLineKind::Normal => Style::default(),
                    };
                    Line::from(Span::styled(dl.content.clone(), style))
                })
                .collect();

            let diff_widget = Paragraph::new(diff_text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Diff (Enter to load, j/k to scroll)")
                        .border_style(diff_block_style),
                )
                .scroll((app.diff_scroll, 0)); // vertical scroll, no horizontal

            frame.render_widget(diff_widget, chunks[1]);

            // --- Status bar ---
            let status = match app.focused_panel {
                Panel::CommitList => Paragraph::new(" j/k: navigate  Enter: view diff  q: quit"),
                Panel::Diff => Paragraph::new(" j/k: scroll  Tab: back to commits  q: quit"),
            };
            frame.render_widget(status, chunks[2]);
        })?;

        if let Event::Key(key) = event::read()? {
            match (&app.focused_panel, key.code) {
                // Commit list controls
                (Panel::CommitList, KeyCode::Char('q')) => break,
                (Panel::CommitList, KeyCode::Char('j') | KeyCode::Down) => app.move_down(),
                (Panel::CommitList, KeyCode::Char('k') | KeyCode::Up) => app.move_up(),
                (Panel::CommitList, KeyCode::Enter) => {
                    app.load_diff(&repo)?;
                }

                // Diff panel controls
                (Panel::Diff, KeyCode::Char('q')) => break,
                (Panel::Diff, KeyCode::Char('j') | KeyCode::Down) => app.scroll_diff_down(),
                (Panel::Diff, KeyCode::Char('k') | KeyCode::Up) => app.scroll_diff_up(),
                (Panel::Diff, KeyCode::Tab) => {
                    app.focused_panel = Panel::CommitList;
                }

                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn load_commits(repo: &Repository) -> Result<(Vec<String>, Vec<String>)> {
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    let mut display = vec![];
    let mut ids = vec![];

    for id in revwalk.take(20) {
        let id = id?;
        let commit = repo.find_commit(id)?;
        let message = commit.summary().unwrap_or("(no message)").to_string();
        let short_id = &id.to_string()[..7];
        display.push(format!("{} {}", short_id, message));
        ids.push(id.to_string());
    }

    Ok((display, ids))
}
