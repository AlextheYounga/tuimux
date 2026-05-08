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

fn maybe_relaunch_outside_tmux() -> Result<bool> {
    if env::var_os("TMUX").is_none() || env::var_os(TUIMUX_RELAUNCHED).is_some() {
        return Ok(false);
    }

    let output = Command::new("tmux")
        .args(["detach-client", "-E", "TUIMUX_RELAUNCHED=1 tuimux"])
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
