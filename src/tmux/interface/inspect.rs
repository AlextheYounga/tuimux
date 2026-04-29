use std::process::{self, Command};

use anyhow::{Context, Result};

use crate::tmux::session::{Pane, Session, Window};

const TMUX_FIELD_SEPARATOR: &str = "\x1f";
const TMUX_LINE_SEPARATOR: &str = "\n";

/// Retrives a [`Session`] by name, or infer the current session if a name is
/// not provided.
///
/// # Errors
/// Returns an error when tmux commands or parsing fail.
pub fn get_session(session_name: Option<&str>) -> Result<Session> {
    let name = if let Some(name) = session_name { name.to_string() } else { get_session_name()? };
    let path =
        get_session_path(&name).with_context(|| format!("Failed to get working directory for session '{name}'"))?;
    let windows = get_windows(&name).with_context(|| format!("Failed to get windows for session '{name}'"))?;
    Ok(Session { name, work_dir: path, windows })
}

/// Captures text output from the active pane in a tmux target.
///
/// # Errors
/// Returns an error when tmux capture-pane fails.
pub fn capture_preview(session_name: &str, window_index: Option<&str>) -> Result<String> {
    let target = match window_index {
        Some(index) => format!("{session_name}:{index}"),
        None => session_name.to_string(),
    };
    let pane_target = resolve_preview_pane(&target)?;

    let output = Command::new("tmux")
        .args(["capture-pane", "-p", "-S", "-200", "-t", &pane_target])
        .output()
        .with_context(|| format!("Failed to capture pane output for target {pane_target}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("tmux capture-pane failed for {pane_target}: {stderr}");
    }

    let output = String::from_utf8(output.stdout).context("Failed to convert tmux capture-pane output to UTF-8")?;
    Ok(output.trim_end().to_string())
}

/// Gets the name of the current tmux session.
///
/// # Errors
/// Returns an error if tmux fails to execute or output parsing fails.
pub fn get_session_name() -> Result<String> {
    let output = Command::new("tmux")
        .arg("display-message")
        .arg("-p")
        .args(["-F", "#{session_name}"])
        .output()
        .context("Failed to execute 'tmux display-message'")?;

    let string_output = String::from_utf8(output.stdout).context("Failed to convert tmux output to UTF-8 string")?;
    Ok(string_output.trim().to_string())
}

/// Lists all existing tmux sessions.
///
/// # Errors
/// Returns an error if tmux commands fail.
pub fn list_sessions() -> Result<Vec<String>> {
    let output = Command::new("tmux")
        .arg("list-sessions")
        .args(["-F", "#{session_name}"])
        .output()
        .context("Failed to get active sessions")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("no server running") {
            return Ok(Vec::new());
        }
        anyhow::bail!("tmux list-sessions failed: {stderr}");
    }

    let string_output = String::from_utf8(output.stdout).context("Failed to convert tmux output to UTF-8 string")?;
    Ok(string_output
        .split(TMUX_LINE_SEPARATOR)
        .filter(|line| !line.trim().is_empty())
        .map(|value| value.trim().to_string())
        .collect())
}

fn resolve_preview_pane(target: &str) -> Result<String> {
    let output = Command::new("tmux")
        .args(["list-panes", "-t", target, "-F", "#{pane_id}"])
        .output()
        .with_context(|| format!("Failed to list panes for target {target}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("tmux list-panes failed for {target}: {stderr}");
    }

    let output = String::from_utf8(output.stdout).context("Failed to convert tmux list-panes output to UTF-8")?;
    let pane_id = output
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .ok_or_else(|| anyhow::anyhow!("No panes found for target {target}"))?;

    Ok(pane_id.to_string())
}

fn get_session_path(session_name: &str) -> Result<String> {
    let output = Command::new("tmux")
        .arg("display-message")
        .arg("-p")
        .args(["-t", session_name])
        .args(["-F", "#{session_path}"])
        .output()
        .context("Failed to execute 'tmux display-message'")?;

    let string_output = String::from_utf8(output.stdout).context("Failed to convert tmux output to UTF-8 string")?;
    Ok(string_output.trim().to_string())
}

fn get_windows(session_name: &str) -> Result<Vec<Window>> {
    let output = Command::new("tmux")
        .arg("list-windows")
        .args(["-t", session_name])
        .args(["-F", "#{window_index}\x1f#{window_name}\x1f#{window_layout}"])
        .output()
        .context("Failed to execute 'tmux list-windows'")?;

    let string_output = String::from_utf8(output.stdout).context("Failed to convert tmux output to UTF-8 string")?;
    string_output
        .split(TMUX_LINE_SEPARATOR)
        .filter(|window| !window.trim().is_empty())
        .map(|window| parse_window_string(window, session_name))
        .collect()
}

fn parse_window_string(window: &str, session_name: &str) -> Result<Window> {
    let mut parts = window.splitn(3, TMUX_FIELD_SEPARATOR);
    match (parts.next(), parts.next(), parts.next()) {
        (Some(index), Some(name), Some(layout)) => {
            let index = index.to_string();
            let window_target = format!("{session_name}:{index}");
            let panes = get_panes(&window_target)?;
            Ok(Window { index, name: name.to_string(), layout: layout.to_string(), panes })
        }
        _ => anyhow::bail!("Failed to parse window string: {window}"),
    }
}

fn get_panes(window_target: &str) -> Result<Vec<Pane>> {
    let output = Command::new("tmux")
        .arg("list-panes")
        .args(["-t", window_target])
        .args(["-F", "#{pane_index}\x1f#{pane_pid}\x1f#{pane_current_path}"])
        .output()
        .with_context(|| format!("Failed to execute 'tmux list-panes' for window {window_target}"))?;

    let string_output = String::from_utf8(output.stdout).context("Failed to convert tmux output to UTF-8 string")?;
    string_output.split(TMUX_LINE_SEPARATOR).filter(|pane| !pane.trim().is_empty()).map(parse_pane_string).collect()
}

fn parse_pane_string(pane: &str) -> Result<Pane> {
    let mut parts = pane.splitn(3, TMUX_FIELD_SEPARATOR);
    match (parts.next(), parts.next(), parts.next()) {
        (Some(index), Some(pid), Some(work_dir_str)) => {
            let process = get_foreground_process(pid)?;
            let current_command = match process {
                Some((cmd_pid, cmdline)) if process::id() != cmd_pid => Some(cmdline),
                _ => None,
            };

            Ok(Pane { index: index.to_string(), current_command, work_dir: work_dir_str.to_string() })
        }
        _ => anyhow::bail!("Failed to parse pane string: {pane}"),
    }
}

fn get_foreground_process(shell_pid: &str) -> Result<Option<(u32, String)>> {
    Ok(get_process_children(shell_pid)?.into_iter().next())
}

fn get_process_children(shell_pid: &str) -> Result<Vec<(u32, String)>> {
    let output = Command::new("ps")
        .args(["-o", "pid=,args="])
        .args(["--ppid", shell_pid])
        .output()
        .with_context(|| format!("Failed to get children of process #{shell_pid}"))?;

    let output_str = String::from_utf8(output.stdout)?;
    let mut children = Vec::new();
    for line in output_str.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some((pid_str, cmdline)) = trimmed.split_once(' ')
            && let Ok(pid) = pid_str.trim().parse::<u32>()
        {
            children.push((pid, cmdline.trim().to_string()));
        }
    }

    Ok(children)
}
