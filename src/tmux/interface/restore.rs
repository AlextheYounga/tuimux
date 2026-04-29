use std::borrow::Cow;
use std::fmt::Write;
use std::fs::write;
use std::process::{self, Command};

use anyhow::{Context, Result};
use shell_escape::escape;
use tempfile::NamedTempFile;

use crate::tmux::interface::attach_to_session;
use crate::tmux::session::{Session, Window};

/// Restores a tmux session from a [`Session`] struct.
///
/// # Errors
/// Returns an error if any tmux command fails, or if writing the temporary
/// restoration script fails.
pub fn restore_session(session: &Session) -> Result<()> {
    if session.windows.is_empty() {
        anyhow::bail!("Cannot restore session without windows");
    }

    let temp_session_name = format!("tsman-temp-{}", process::id());
    let mut script_str = String::new();

    writeln!(script_str, "tmux new-session -d -s {} -c {}", temp_session_name, escape(Cow::from(&session.work_dir)))?;

    let first_window = &session.windows[0];
    script_str += &window_config_script(&temp_session_name, session, first_window)?;

    for window in session.windows.iter().skip(1) {
        writeln!(
            script_str,
            "tmux new-window -d -t {} -c {}",
            temp_session_name,
            escape(Cow::from(&session.work_dir))
        )?;
        script_str += &window_config_script(&temp_session_name, session, window)?;
    }

    writeln!(script_str, "tmux rename-session -t {} {}", temp_session_name, session.name)?;

    let script = NamedTempFile::new()?;
    write(script.path(), script_str)?;
    Command::new("sh").arg(script.path()).status().context("Failed to reconstruct session")?;
    attach_to_session(&session.name)
}

fn window_config_script(temp_session_name: &str, session: &Session, window: &Window) -> Result<String> {
    if window.panes.is_empty() {
        anyhow::bail!("Cannot restore window '{}' without panes", window.name);
    }

    let window_target = format!("{}:{}", temp_session_name, window.index);
    let mut cmd = String::new();

    writeln!(cmd, "tmux rename-window -t {} {}", window_target, window.name)?;
    for _ in window.panes.iter().skip(1) {
        writeln!(cmd, "tmux split-window -d -t {} -c {}", window_target, escape(Cow::from(&session.work_dir)))?;
    }

    writeln!(cmd, "tmux select-layout -t {} {}", window_target, escape(Cow::from(&window.layout)))?;

    for pane in &window.panes {
        let pane_target = format!("{}.{}", window_target, pane.index);
        if pane.work_dir != session.work_dir {
            writeln!(
                cmd,
                "tmux send-keys -t {} {} C-m",
                pane_target,
                escape(format!("cd {}; clear", escape(Cow::from(&pane.work_dir))).into()),
            )?;
        }

        if let Some(pane_cmd) = &pane.current_command {
            writeln!(cmd, "tmux send-keys -t {} {} C-m", pane_target, escape(pane_cmd.into()))?;
        }
    }

    Ok(cmd)
}
