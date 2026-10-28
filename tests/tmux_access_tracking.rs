use std::env;
use std::fs;
use std::iter::once;
use std::path::PathBuf;
use std::process::Command;

use tuimux::tmux::interface::{attach_to_session, attach_to_window};

const HELPER_ENV: &str = "TUIMUX_ACCESS_TEST_HELPER";
const LOG_ENV: &str = "TUIMUX_TMUX_ACCESS_LOG";

#[test]
fn attaching_through_tuimux_records_session_access() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let log_path = temp_dir.path().join("tmux.log");
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tmux-access");
    let path = env::join_paths(once(fixture_dir).chain(env::split_paths(&env::var_os("PATH").unwrap_or_default())))?;

    let output = Command::new(env::current_exe()?)
        .args(["--exact", "tmux_access_helper", "--ignored"])
        .env(HELPER_ENV, "1")
        .env(LOG_ENV, &log_path)
        .env("PATH", path)
        .env_remove("TMUX")
        .output()?;

    assert!(
        output.status.success(),
        "helper test failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    Ok(())
}

#[test]
#[ignore = "run in a subprocess with the tmux access fixture"]
fn tmux_access_helper() -> anyhow::Result<()> {
    assert_eq!(env::var_os(HELPER_ENV).as_deref(), Some("1".as_ref()));

    attach_to_session("alpha")?;
    attach_to_window("beta", "2")?;

    let log = fs::read_to_string(env::var(LOG_ENV)?)?;
    let commands: Vec<&str> = log.lines().collect();
    assert_eq!(commands.len(), 2);
    assert_access_command(commands[0], "attach-session -t alpha", "alpha")?;
    assert_access_command(commands[1], "attach-session -t beta ; select-window -t beta:2", "beta")?;

    Ok(())
}

fn assert_access_command(command: &str, expected_prefix: &str, session_name: &str) -> anyhow::Result<()> {
    let expected_prefix = format!("{expected_prefix} ; set-option -t {session_name} @tuimux_last_accessed ");
    let timestamp =
        command.strip_prefix(&expected_prefix).ok_or_else(|| anyhow::anyhow!("unexpected tmux command: {command}"))?;
    assert!(timestamp.parse::<u64>().is_ok(), "invalid access timestamp: {timestamp}");

    Ok(())
}
