// src/git.rs
use crate::app::{ChangedFile, CommitInfo, DiffLine, DiffLineKind};
use git2::{DiffFormat, DiffOptions, Oid, Repository};

pub struct GitWorker {
    repo: Repository,
}

impl GitWorker {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        let repo = Repository::open(path)?;
        Ok(Self { repo })
    }

    pub fn get_commits(&self, limit: usize) -> anyhow::Result<Vec<CommitInfo>> {
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;

        let mut commits = Vec::new();

        for id in revwalk.take(limit) {
            let id = id?;
            let commit = self.repo.find_commit(id)?;
            let summary = commit.summary().unwrap_or("(no message)").to_string();
            let short_id = id.to_string()[..7].to_string();
            commits.push(CommitInfo {
                id: id.to_string(),
                short_id,
                summary,
            });
        }
        Ok(commits)
    }

    pub fn get_commit_diffs(&self, commit_hash: &str) -> anyhow::Result<Vec<ChangedFile>> {
        let id = Oid::from_str(commit_hash)?;
        let commit = self.repo.find_commit(id)?;

        let commit_tree = commit.tree()?;
        let parent_tree = commit.parents().next().and_then(|p| p.tree().ok());

        let mut opts = DiffOptions::new();
        let diff = self.repo.diff_tree_to_tree(
            parent_tree.as_ref(),
            Some(&commit_tree),
            Some(&mut opts),
        )?;

        let mut files = Vec::new();
        let mut current_file_path: Option<String> = None;
        let mut current_lines = Vec::new();
        // First pass: collect unique file paths from the diff
        diff.print(DiffFormat::Patch, |delta, _hunk, line| {
            let path = delta
                .new_file()
                .path()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            if let Some(p) = current_file_path.clone()
                && p != path
            {
                files.push(ChangedFile {
                    path: p.clone(),
                    diff_lines: std::mem::take(&mut current_lines),
                });
            }
            current_file_path = Some(path);
            let content = std::str::from_utf8(line.content())
                .unwrap_or("")
                .trim_end_matches(['\n', '\r'])
                .to_string();
            let kind = match line.origin() {
                '+' => DiffLineKind::Added,
                '-' => DiffLineKind::Removed,
                'H' => DiffLineKind::Header,
                _ => DiffLineKind::Normal,
            };
            current_lines.push(DiffLine { content, kind });
            true
        })?;
        if let Some(path) = current_file_path {
            files.push(ChangedFile {
                path,
                diff_lines: current_lines,
            });
        }
        Ok(files)
    }
}
