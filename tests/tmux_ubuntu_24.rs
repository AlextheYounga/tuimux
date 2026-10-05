use std::env;
use std::iter::once;
use std::path::PathBuf;
use std::process::Command;

use tuimux::tmux::interface::fetch_all_sessions;

const HELPER_ENV: &str = "TUIMUX_TMUX_3_4_TEST_HELPER";

#[test]
fn fetch_all_sessions_parses_tmux_3_4_escaped_field_separators() -> anyhow::Result<()> {
    let test_binary = env::current_exe()?;
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tmux-3.4");
    let path = env::join_paths(once(fixture_dir).chain(env::split_paths(&env::var_os("PATH").unwrap_or_default())))?;

    let output = Command::new(test_binary)
        .args(["--exact", "tmux_3_4_fetch_helper", "--ignored"])
        .env(HELPER_ENV, "1")
        .env("PATH", path)
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
#[ignore = "run in a subprocess with the tmux 3.4 fixture"]
fn tmux_3_4_fetch_helper() -> anyhow::Result<()> {
    assert_eq!(env::var_os(HELPER_ENV).as_deref(), Some("1".as_ref()));

    let sessions = fetch_all_sessions()?;

    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].name, "ubuntu");
    assert_eq!(sessions[0].windows.len(), 1);
    assert_eq!(sessions[0].windows[0].name, "bash");

    Ok(())
}
