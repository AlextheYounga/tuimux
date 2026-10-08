pub mod app;
pub mod tmux;
pub mod ui;

use std::env;
use std::process::Command;

use anyhow::{Context, Result};

const TUIMUX_RELAUNCHED: &str = "TUIMUX_RELAUNCHED";

/// Starts the tuimux application.
///
/// # Errors
/// Returns an error when the app runtime fails.
pub fn run() -> Result<()> {
    if maybe_relaunch_outside_tmux()? {
        return Ok(());
    }

    let mut app = app::App::new();
    app.run()
}

/// Detaches the current tmux client and relaunches tuimux outside tmux if running inside a session.
///
/// # Errors
/// Returns an error if the current executable path cannot be determined or if `tmux detach-client` fails.
pub fn maybe_relaunch_outside_tmux() -> Result<bool> {
    if env::var_os("TMUX").is_none() || env::var_os(TUIMUX_RELAUNCHED).is_some() {
        return Ok(false);
    }

    let current_exe = env::current_exe()
        .context("Failed to determine current executable path for relaunch")?;
    let exe = shell_escape::escape(current_exe.to_string_lossy());
    let relaunch_cmd = format!("TUIMUX_RELAUNCHED=1 {exe}");

    let output = Command::new("tmux")
        .args(["detach-client", "-E", &relaunch_cmd])
        .output()
        .context("Failed to relaunch tuimux outside tmux. tmux sessions should be nested with care.")?;

    if output.status.success() {
        return Ok(true);
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        anyhow::bail!("Failed to relaunch tuimux outside tmux. tmux sessions should be nested with care.");
    }

    anyhow::bail!(
        "Failed to relaunch tuimux outside tmux. tmux sessions should be nested with care. tmux said: {stderr}"
    );
}
