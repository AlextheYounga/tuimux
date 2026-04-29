use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::tmux::session::Session;

const BACKUP_RELATIVE_PATH: &str = ".config/tuimux/tuimux-sessions.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionBackupFile {
    sessions: Vec<SessionBackup>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionBackup {
    pub name: String,
    pub path: String,
    pub windows: Vec<WindowBackup>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowBackup {
    pub name: String,
    pub path: String,
}

impl SessionBackupFile {
    #[must_use]
    pub fn from_sessions(sessions: &[Session]) -> Self {
        let session_backups = sessions
            .iter()
            .map(|session| SessionBackup {
                name: session.name.clone(),
                path: session.work_dir.clone(),
                windows: session
                    .windows
                    .iter()
                    .map(|window| WindowBackup {
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
    pub fn sessions(&self) -> &[SessionBackup] {
        &self.sessions
    }
}

pub fn backup_path() -> Result<PathBuf> {
    let home = env::var("HOME").context("HOME environment variable is not set")?;
    Ok(PathBuf::from(home).join(BACKUP_RELATIVE_PATH))
}

pub fn export_sessions(sessions: &[Session]) -> Result<PathBuf> {
    let path = backup_path()?;
    let parent =
        path.parent().ok_or_else(|| anyhow::anyhow!("Backup path has no parent directory: {}", path.display()))?;

    fs::create_dir_all(parent).with_context(|| format!("Failed to create backup directory: {}", parent.display()))?;

    let backup = SessionBackupFile::from_sessions(sessions);
    let payload = serde_json::to_string_pretty(&backup).context("Failed to serialize session backup")?;
    fs::write(&path, payload).with_context(|| format!("Failed to write backup file: {}", path.display()))?;

    Ok(path)
}

pub fn import_sessions() -> Result<SessionBackupFile> {
    let path = backup_path()?;
    let payload =
        fs::read_to_string(&path).with_context(|| format!("Failed to read backup file: {}", path.display()))?;
    let backup: SessionBackupFile = serde_json::from_str(&payload).context("Failed to parse backup JSON")?;
    Ok(backup)
}
