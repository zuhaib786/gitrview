mod app;
mod git;
mod ui;

use anyhow::Result;
use app::{Action, App, Panel};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use git::GitWorker;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::stdout;

fn main() -> Result<()> {
    // ── Setup Terminal ────────────────────────────────────────────────────────
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // ── Initialize App State ──────────────────────────────────────────────────
    let git_worker = GitWorker::new(".")?;

    // Fetch initial commits to populate the list
    let commits = git_worker.get_commits(50)?;

    // Create our state machine
    let mut app = App {
        active_panel: Panel::CommitList,
        git_worker,
        should_quit: false,
        commits,
        commit_list_state: Default::default(),
        changed_files: vec![],
        file_list_state: Default::default(),
        diff_scroll: 0,
    };

    // Select the first commit by default
    app.commit_list_state.select(Some(0));

    // ── Main Event Loop ───────────────────────────────────────────────────────
    loop {
        // 1. Draw the UI based on the current App state
        terminal.draw(|frame| {
            ui::render(frame, &mut app);
        })?;

        // 2. Exit condition
        if app.should_quit {
            break;
        }

        // 3. Handle Keyboard Input
        if let Event::Key(key) = event::read()? {
            // Map physical keys to domain Actions
            let action = match (&app.active_panel, key.code) {
                // Global keys
                (_, KeyCode::Char('q')) => Some(Action::Quit),
                (_, KeyCode::Tab) => Some(Action::FocusNext),
                (_, KeyCode::Esc) => Some(Action::FocusPrev),

                // Navigation keys
                (_, KeyCode::Char('j') | KeyCode::Down) => Some(Action::MoveDown),
                (_, KeyCode::Char('k') | KeyCode::Up) => Some(Action::MoveUp),

                // Context-specific keys
                (Panel::CommitList, KeyCode::Enter) => Some(Action::SelectCommit),

                // Ignore anything else
                _ => None,
            };

            // 4. Update the App state
            if let Some(action) = action {
                app.update(action);

                // --- Side Effect Handling ---
                // If the user selected a commit, we need to fetch the actual file diffs.
                // Because git fetching returns a Result (it can fail), it's often safer
                // to handle that IO side-effect here in main.rs rather than hiding failures inside app.rs.
                if key.code == KeyCode::Enter
                    && app.active_panel == Panel::CommitList
                    && let Some(idx) = app.commit_list_state.selected()
                    && let Some(commit) = app.commits.get(idx)
                {
                    match app.git_worker.get_commit_diffs(&commit.id) {
                        Ok(files) => {
                            app.changed_files = files;
                            app.file_list_state.select(Some(0)); // Auto-select first file
                            app.active_panel = Panel::FileTree; // Auto-focus file tree
                            app.diff_scroll = 0;
                        }
                        Err(e) => {
                            // In a full app, you'd store this error in app.state
                            // and show it in a popup widget!
                            eprintln!("Error loading diff: {}", e);
                        }
                    }
                }
            }
        }
    }

    // ── Teardown Terminal ─────────────────────────────────────────────────────
    // If the app panics before this line, the terminal will be left in a broken state.
    // In production Rust TUI apps, it's highly recommended to use a panic hook
    // to ensure these lines always run.
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
