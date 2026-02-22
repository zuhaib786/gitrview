use crate::git::GitWorker;
use ratatui::widgets::ListState;
#[derive(PartialEq)]
pub enum Panel {
    CommitList,
    FileTree,
    Diff,
}

#[derive(PartialEq, Clone)]
pub enum DiffLineKind {
    Added,
    Removed,
    Header,
    Normal,
}
#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub id: String,
    pub short_id: String,
    pub summary: String,
}

#[derive(Clone)]
pub struct DiffLine {
    pub content: String,
    pub kind: DiffLineKind,
}

// A changed file inside a commit
pub struct ChangedFile {
    pub path: String,
    pub diff_lines: Vec<DiffLine>, // pre-computed diff for this file only
}
pub struct SbsRow {
    pub left: Option<DiffLine>,
    pub right: Option<DiffLine>,
}

pub enum Action {
    MoveUp,
    MoveDown,
    FocusNext,
    FocusPrev,
    SelectCommit,
    Quit,
}

pub struct App {
    pub active_panel: Panel,
    pub git_worker: GitWorker,
    pub should_quit: bool,

    pub commits: Vec<CommitInfo>,
    pub commit_list_state: ListState,

    pub changed_files: Vec<ChangedFile>,
    pub file_list_state: ListState,

    pub diff_scroll: u16,
}

pub fn build_side_by_side(lines: &[DiffLine]) -> Vec<SbsRow> {
    let mut rows = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = &lines[i];
        match line.kind {
            DiffLineKind::Normal | DiffLineKind::Header => {
                rows.push(SbsRow {
                    left: Some(line.clone()),
                    right: Some(line.clone()),
                });
                i += 1;
            }
            DiffLineKind::Removed => {
                if i + 1 < lines.len() && lines[i + 1].kind == DiffLineKind::Added {
                    rows.push(SbsRow {
                        left: Some(line.clone()),
                        right: Some(lines[i + 1].clone()),
                    });
                    i += 2;
                } else {
                    rows.push(SbsRow {
                        left: Some(line.clone()),
                        right: None,
                    });
                    i += 1;
                }
            }
            DiffLineKind::Added => {
                rows.push(SbsRow {
                    left: None,
                    right: Some(line.clone()),
                });
                i += 1;
            }
        }
    }

    rows
}

impl App {
    pub fn update(&mut self, action: Action) {
        match action {
            Action::MoveUp => self.handle_move_up(),
            Action::MoveDown => self.handle_move_down(),
            Action::FocusNext => self.handle_focus_next(),
            Action::FocusPrev => self.handle_focus_prev(),
            Action::SelectCommit => self.handle_select_commit(),
            Action::Quit => self.should_quit = true,
        };
    }
    fn handle_move_up(&mut self) {
        match self.active_panel {
            Panel::CommitList => self.commit_up(),
            Panel::FileTree => self.file_up(),
            Panel::Diff => self.diff_up(),
        }
    }
    fn handle_move_down(&mut self) {
        match self.active_panel {
            Panel::CommitList => self.commit_down(),
            Panel::FileTree => self.file_down(),
            Panel::Diff => self.diff_down(),
        }
    }
    fn handle_focus_next(&mut self) {
        self.active_panel = match self.active_panel {
            Panel::CommitList => Panel::FileTree,
            Panel::FileTree => Panel::Diff,
            Panel::Diff => Panel::CommitList,
        };
    }
    fn handle_focus_prev(&mut self) {
        self.active_panel = match self.active_panel {
            Panel::Diff => Panel::FileTree,
            Panel::FileTree => Panel::CommitList,
            Panel::CommitList => Panel::CommitList,
        };
    }
    fn handle_select_commit(&mut self) {
        if let Ok(files) = self.git_worker.get_commit_diffs(&self.current_id()) {
            self.changed_files = files;
            self.active_panel = Panel::FileTree;
        }
    }
    // --- Commits -------------------------------
    fn commit_up(&mut self) {
        // Fixed: Reading from commit_list_state instead of git_worker
        let current = self.commit_list_state.selected().unwrap_or(0);
        self.commit_list_state
            .select(Some(current.saturating_sub(1)));
    }
    fn commit_down(&mut self) {
        let current = self.commit_list_state.selected().unwrap_or(0);
        let next = (current + 1).min(self.commits.len().saturating_sub(1));
        self.commit_list_state.select(Some(next));
    }
    fn current_id(&self) -> String {
        let id = self.commit_list_state.selected().unwrap_or(0);
        self.commits[id].id.clone()
    }
    // --- Files -------------------------------
    fn file_up(&mut self) {
        let current = self.file_list_state.selected().unwrap_or(0);
        self.file_list_state.select(Some(current.saturating_sub(1)));
        self.diff_scroll = 0; // Reset scroll on change
    }

    fn file_down(&mut self) {
        let current = self.file_list_state.selected().unwrap_or(0);
        let next = (current + 1).min(self.changed_files.len().saturating_sub(1));
        self.file_list_state.select(Some(next));
        self.diff_scroll = 0; // Reset scroll on change
    }
    // --- Diffs -------------------------------
    fn diff_up(&mut self) {
        self.diff_scroll = self.diff_scroll.saturating_sub(3);
    }

    fn diff_down(&mut self) {
        self.diff_scroll = self.diff_scroll.saturating_add(3);
    }
}
