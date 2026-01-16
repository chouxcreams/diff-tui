use anyhow::{Context, Result};
use git2::{Repository as Git2Repository, StatusOptions};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileStatus {
    Modified,
    Added,
    Deleted,
    Renamed,
    Untracked,
}

#[derive(Debug, Clone)]
pub struct ChangedFile {
    pub path: String,
    pub status: FileStatus,
}

pub struct Repository {
    inner: Git2Repository,
}

impl Repository {
    pub fn open_current_dir() -> Result<Self> {
        let repo = Git2Repository::discover(".").context(
            "Failed to find git repository. Please run this command inside a git repository.",
        )?;
        Ok(Self { inner: repo })
    }

    pub fn get_changed_files(&self) -> Result<Vec<ChangedFile>> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(true)
            .include_ignored(false);

        let statuses = self
            .inner
            .statuses(Some(&mut opts))
            .context("Failed to get repository status")?;

        let mut files = Vec::new();

        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("").to_string();
            let status = entry.status();

            let file_status = if status.is_index_new() || status.is_wt_new() {
                FileStatus::Added
            } else if status.is_index_deleted() || status.is_wt_deleted() {
                FileStatus::Deleted
            } else if status.is_index_renamed() || status.is_wt_renamed() {
                FileStatus::Renamed
            } else if status.is_index_modified() || status.is_wt_modified() {
                FileStatus::Modified
            } else if status.is_wt_new() {
                FileStatus::Untracked
            } else {
                continue;
            };

            files.push(ChangedFile {
                path,
                status: file_status,
            });
        }

        files.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(files)
    }
}
