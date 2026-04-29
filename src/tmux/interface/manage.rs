use std::env;
use std::process::Command;

use anyhow::{Context, Result};

/// Checks if a tmux session exists.
///
/// # Arguments
/// * `session_name` - The name of the tmux session.
///
/// # Returns
/// `Ok(true)` if the session exists, `Ok(false)` otherwise.
///
/// # Errors
/// Returns an error if the `tmux list-session` command fails.
pub fn session_exists(session_name: &str) -> Result<bool> {
    let output = Command::new("tmux")
        .arg("list-session")
        .args(["-F", "#{session_name}"])
        .output()
        .context("Failed to get sessions")?;

    let output_str = String::from_utf8(output.stdout)?;
    let session_names = output_str.split('\n').collect::<Vec<&str>>();

    Ok(session_names.contains(&session_name))
}

/// Attaches to or switches to a tmux session.
///
/// # Errors
/// Returns an error if the tmux attach/switch command fails.
pub fn attach_to_session(session_name: &str) -> Result<()> {
    let is_attached = env::var("TMUX").is_ok();
    let attach_cmd = if is_attached { "switch-client" } else { "attach-session" };

    Command::new("tmux").arg(attach_cmd).args(["-t", session_name]).status().context("Failed to attach session")?;

    Ok(())
}

/// Attaches to a tmux session and selects a specific window.
///
/// # Errors
/// Returns an error if tmux fails to switch/attach or select the target window.
pub fn attach_to_window(session_name: &str, window_index: &str) -> Result<()> {
    let window_target = format!("{session_name}:{window_index}");
    let is_attached = env::var("TMUX").is_ok();

    let status = if is_attached {
        Command::new("tmux")
            .args(["switch-client", "-t", session_name])
            .args([";", "select-window", "-t", &window_target])
            .status()
            .context("Failed to switch to target window")?
    } else {
        Command::new("tmux")
            .args(["attach-session", "-t", session_name])
            .args([";", "select-window", "-t", &window_target])
            .status()
            .context("Failed to attach to target window")?
    };

    if !status.success() {
        anyhow::bail!("tmux failed to attach/select window {window_target}");
    }

    Ok(())
}

/// Creates a detached tmux session.
///
/// # Errors
/// Returns an error if `tmux new-session` fails.
pub fn create_session(session_name: &str) -> Result<()> {
    create_session_with_path(session_name, "")
}

/// Creates a detached tmux session at a specific working directory.
///
/// # Errors
/// Returns an error if `tmux new-session` fails.
pub fn create_session_with_path(session_name: &str, work_dir: &str) -> Result<()> {
    let mut command = Command::new("tmux");
    command.args(["new-session", "-d", "-s", session_name]);

    if !work_dir.is_empty() {
        command.args(["-c", work_dir]);
    }

    let status = command.status().context("Failed to create session")?;
    if !status.success() {
        anyhow::bail!("tmux failed to create session {session_name}");
    }

    Ok(())
}

/// Creates a new window in a target session.
///
/// # Errors
/// Returns an error if `tmux new-window` fails.
pub fn create_window(session_name: &str, window_name: &str) -> Result<()> {
    create_window_with_path(session_name, window_name, "")
}

/// Creates a new window in a target session at a specific path.
///
/// # Errors
/// Returns an error if `tmux new-window` fails.
pub fn create_window_with_path(session_name: &str, window_name: &str, work_dir: &str) -> Result<()> {
    let target_session = format!("{session_name}:");
    let mut command = Command::new("tmux");
    command.args(["new-window", "-t", &target_session, "-n", window_name]);

    if !work_dir.is_empty() {
        command.args(["-c", work_dir]);
    }

    let status = command.status().context("Failed to create window")?;
    if !status.success() {
        anyhow::bail!("tmux failed to create window {window_name} in {session_name}");
    }

    Ok(())
}

/// Renames a tmux session.
///
/// # Errors
/// Returns an error if `tmux rename-session` fails.
pub fn rename_session(session_name: &str, new_name: &str) -> Result<()> {
    Command::new("tmux")
        .arg("rename-session")
        .args(["-t", session_name])
        .arg(new_name)
        .status()
        .context("Failed to rename session")?;
    Ok(())
}

/// Renames a tmux window.
///
/// # Errors
/// Returns an error if `tmux rename-window` fails.
pub fn rename_window(session_name: &str, window_index: &str, new_name: &str) -> Result<()> {
    let window_target = format!("{session_name}:{window_index}");
    let status = Command::new("tmux")
        .arg("rename-window")
        .args(["-t", &window_target])
        .arg(new_name)
        .status()
        .context("Failed to rename window")?;

    if !status.success() {
        anyhow::bail!("tmux failed to rename window {window_target}");
    }

    Ok(())
}

/// Closes a tmux session by name.
///
/// # Errors
/// Returns an error if `tmux kill-session` fails.
pub fn close_session(session_name: &str) -> Result<()> {
    Command::new("tmux").arg("kill-session").args(["-t", session_name]).status().context("Failed to kill session")?;
    Ok(())
}

/// Closes a tmux window by session and index.
///
/// # Errors
/// Returns an error if `tmux kill-window` fails.
pub fn close_window(session_name: &str, window_index: &str) -> Result<()> {
    let window_target = format!("{session_name}:{window_index}");
    let status = Command::new("tmux")
        .arg("kill-window")
        .args(["-t", &window_target])
        .status()
        .context("Failed to kill window")?;

    if !status.success() {
        anyhow::bail!("tmux failed to kill window {window_target}");
    }

    Ok(())
}
