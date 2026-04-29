use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::app::App;
use crate::app::state::{ConfirmAction, Modal};
use crate::tmux::session::Session;

const BACKUP_RELATIVE_PATH: &str = ".config/tuimux/tuimux-sessions.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionBackupFile {
    sessions: Vec<SessionRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionRecord {
    pub name: String,
    pub path: String,
    pub windows: Vec<WindowRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowRecord {
    pub name: String,
    pub path: String,
}

impl SessionBackupFile {
    #[must_use]
    pub fn from_sessions(sessions: &[Session]) -> Self {
        let session_backups = sessions
            .iter()
            .map(|session| SessionRecord {
                name: session.name.clone(),
                path: session.work_dir.clone(),
                windows: session
                    .windows
                    .iter()
                    .map(|window| WindowRecord {
                        name: window.name.clone(),
                        path: window
                            .panes
                            .first()
                            .map_or_else(|| session.work_dir.clone(), |pane| pane.work_dir.clone()),
                    })
                    .collect(),
            })
            .collect();

        Self { sessions: session_backups }
    }

    #[must_use]
    pub fn sessions(&self) -> &[SessionRecord] {
        &self.sessions
    }
}

/// Returns the configured backup file path in the user's home directory.
///
/// # Errors
/// Returns an error when `HOME` is not set.
pub fn sessions_file_path() -> Result<PathBuf> {
    let home = env::var("HOME").context("HOME environment variable is not set")?;
    Ok(PathBuf::from(home).join(BACKUP_RELATIVE_PATH))
}

/// Checks whether the session backup file already exists.
///
/// # Errors
/// Returns an error when `HOME` is not set.
pub fn file_exists() -> Result<bool> {
    let path = sessions_file_path()?;
    Ok(path.exists())
}

/// Exports current sessions/windows into the backup file.
///
/// # Errors
/// Returns an error when creating the backup directory, serializing payload,
/// or writing the file fails.
pub fn write_sessions(sessions: &[Session]) -> Result<PathBuf> {
    let path = sessions_file_path()?;
    let parent =
        path.parent().ok_or_else(|| anyhow::anyhow!("Backup path has no parent directory: {}", path.display()))?;

    fs::create_dir_all(parent).with_context(|| format!("Failed to create backup directory: {}", parent.display()))?;

    let backup = SessionBackupFile::from_sessions(sessions);
    let payload = serde_json::to_string_pretty(&backup).context("Failed to serialize session backup")?;
    fs::write(&path, payload).with_context(|| format!("Failed to write backup file: {}", path.display()))?;

    Ok(path)
}

/// Imports sessions/windows from the backup file.
///
/// # Errors
/// Returns an error when reading the backup file or parsing JSON fails.
pub fn import_sessions() -> Result<SessionBackupFile> {
    let path = sessions_file_path()?;
    let payload =
        fs::read_to_string(&path).with_context(|| format!("Failed to read backup file: {}", path.display()))?;
    let backup: SessionBackupFile = serde_json::from_str(&payload).context("Failed to parse backup JSON")?;
    Ok(backup)
}

impl App {
    pub(super) fn export_sessions(&mut self) {
        match file_exists() {
            Ok(true) => {
                self.state.modal = Some(Modal::Confirm {
                    title: String::from("Overwrite existing export file"),
                    prompt: String::from("Press y/Enter to overwrite, n/Esc to cancel"),
                    action: ConfirmAction::OverwriteSessionExport,
                });
                return;
            }
            Ok(false) => {}
            Err(error) => {
                self.set_error_status(&format!("Export failed: {error}"));
                return;
            }
        }

        self.perform_export();
    }

    pub(super) fn perform_export(&mut self) {
        match write_sessions(&self.state.sessions) {
            Ok(path) => self.set_status(&format!("Exported sessions to {}", path.display())),
            Err(error) => self.set_error_status(&format!("Export failed: {error}")),
        }
    }
}
