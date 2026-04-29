use anyhow::{Context, Result};

use crate::app::App;
use crate::app::backup;
use crate::app::state::{ConfirmAction, Modal};
use crate::tmux::interface::{
    create_session_with_path, create_window_with_path, get_session, rename_window, session_exists,
};

#[derive(Debug, Default, Clone, Copy)]
struct RestoreCounters {
    created_sessions: usize,
    created_windows: usize,
    renamed_sessions: usize,
}

impl App {
    pub(super) fn export_sessions(&mut self) {
        match backup::export_file_exists() {
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
        match backup::export_sessions(&self.state.sessions) {
            Ok(path) => self.set_status(&format!("Exported sessions to {}", path.display())),
            Err(error) => self.set_error_status(&format!("Export failed: {error}")),
        }
    }

    pub(super) fn restore_sessions(&mut self) {
        let backup_file = match backup::import_sessions() {
            Ok(file) => file,
            Err(error) => {
                self.set_error_status(&format!("Restore failed: {error}"));
                return;
            }
        };

        let session_count = backup_file.sessions().len();
        let window_count = backup_file.sessions().iter().map(|session| session.windows.len()).sum::<usize>();
        self.state.modal = Some(Modal::Confirm {
            title: String::from("Confirm restore"),
            prompt: format!(
                "This will restore {session_count} sessions and {window_count} windows. Press y/Enter to continue, n/Esc to cancel"
            ),
            action: ConfirmAction::RunSessionRestore,
        });
    }

    pub(super) fn perform_restore_sessions(&mut self) {
        let backup_file = match backup::import_sessions() {
            Ok(file) => file,
            Err(error) => {
                self.set_error_status(&format!("Restore failed: {error}"));
                return;
            }
        };

        let mut counters = RestoreCounters::default();
        let mut restore_errors: Vec<String> = Vec::new();

        for session_backup in backup_file.sessions() {
            let target_session_name = match Self::resolve_session_name_for_restore(&session_backup.name) {
                Ok(name) => {
                    if name != session_backup.name {
                        counters.renamed_sessions += 1;
                    }
                    name
                }
                Err(error) => {
                    self.set_error_status(&format!(
                        "Restore failed while resolving session {}: {error}",
                        session_backup.name
                    ));
                    return;
                }
            };

            match Self::restore_one_session(session_backup, &target_session_name) {
                Ok(created_windows) => {
                    counters.created_sessions += 1;
                    counters.created_windows += created_windows;
                }
                Err(error) => restore_errors.push(error.to_string()),
            }
        }

        self.refresh_sessions();
        if restore_errors.is_empty() {
            self.set_status(&format!(
                "Restore complete: created {} sessions and {} windows, renamed {} duplicate sessions",
                counters.created_sessions, counters.created_windows, counters.renamed_sessions
            ));
            return;
        }

        let first_error = restore_errors.first().map_or("unknown error", String::as_str);
        self.set_error_status(&format!(
            "Restore finished with {} errors: {}; created {} sessions and {} windows, renamed {} duplicate sessions",
            restore_errors.len(),
            first_error,
            counters.created_sessions,
            counters.created_windows,
            counters.renamed_sessions
        ));
    }

    fn resolve_session_name_for_restore(session_name: &str) -> Result<String> {
        if !session_exists(session_name)? {
            return Ok(session_name.to_string());
        }

        let mut suffix = 2usize;
        loop {
            let candidate = format!("{session_name}-restored-{suffix}");
            if !session_exists(&candidate)? {
                return Ok(candidate);
            }

            suffix += 1;
        }
    }

    fn restore_one_session(session_backup: &backup::SessionRecord, target_session_name: &str) -> Result<usize> {
        if session_backup.windows.is_empty() {
            create_session_with_path(target_session_name, &session_backup.path)
                .with_context(|| format!("session {target_session_name}: create failed"))?;
            return Ok(0);
        }

        let first_window = &session_backup.windows[0];
        create_session_with_path(target_session_name, &first_window.path)
            .with_context(|| format!("session {target_session_name}: create failed"))?;

        let created_session = get_session(Some(target_session_name))
            .with_context(|| format!("session {target_session_name}: read after create failed"))?;

        let first_window_index = if let Some(window) = created_session.windows.first() {
            window.index.as_str()
        } else {
            anyhow::bail!("session {target_session_name}: created with no windows");
        };

        rename_window(target_session_name, first_window_index, &first_window.name)
            .with_context(|| format!("session {target_session_name}: rename first window failed"))?;

        let mut created_windows = 1usize;
        for window_backup in session_backup.windows.iter().skip(1) {
            if let Err(error) = create_window_with_path(target_session_name, &window_backup.name, &window_backup.path) {
                anyhow::bail!("session {target_session_name}: create window {} failed: {error}", window_backup.name);
            }

            created_windows += 1;
        }

        Ok(created_windows)
    }
}
