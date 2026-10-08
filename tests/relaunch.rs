use std::env;
use std::fs;
use std::iter::once;
use std::path::PathBuf;
use std::process::Command;

use tuimux::maybe_relaunch_outside_tmux;

const HELPER_MODE_ENV: &str = "TUIMUX_RELAUNCH_TEST_HELPER_MODE";

#[test]
fn relaunch_noop_when_tmux_environment_variable_is_not_set() -> anyhow::Result<()> {
    let test_binary = env::current_exe()?;
    let output = Command::new(test_binary)
        .args(["--exact", "relaunch_helper", "--ignored"])
        .env(HELPER_MODE_ENV, "noop_no_tmux")
        .env_remove("TMUX")
        .env_remove("TUIMUX_RELAUNCHED")
        .output()?;

    assert!(
        output.status.success(),
        "helper failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    Ok(())
}

#[test]
fn relaunch_noop_when_already_relaunched_even_if_tmux_variable_is_present() -> anyhow::Result<()> {
    let test_binary = env::current_exe()?;
    let output = Command::new(test_binary)
        .args(["--exact", "relaunch_helper", "--ignored"])
        .env(HELPER_MODE_ENV, "noop_already_relaunched")
        .env("TMUX", "/tmp/tmux-1000/default,1234,0")
        .env("TUIMUX_RELAUNCHED", "1")
        .output()?;

    assert!(
        output.status.success(),
        "helper failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    Ok(())
}

#[test]
fn relaunch_invokes_tmux_detach_with_escaped_current_exe_path() -> anyhow::Result<()> {
    let test_binary = env::current_exe()?;
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mock-tmux");
    let path = env::join_paths(once(fixture_dir).chain(env::split_paths(&env::var_os("PATH").unwrap_or_default())))?;

    let temp_dir = tempfile::tempdir()?;
    let log_file = temp_dir.path().join("mock_tmux.log");

    let output = Command::new(&test_binary)
        .args(["--exact", "relaunch_helper", "--ignored"])
        .env(HELPER_MODE_ENV, "relaunch_success")
        .env("TMUX", "/tmp/tmux-1000/default,1234,0")
        .env_remove("TUIMUX_RELAUNCHED")
        .env("PATH", path)
        .env("MOCK_TMUX_LOG", &log_file)
        .output()?;

    assert!(
        output.status.success(),
        "helper failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let logged_args = fs::read_to_string(&log_file)?;
    let escaped_binary = shell_escape::escape(test_binary.to_string_lossy());
    let expected_cmd = format!("detach-client -E TUIMUX_RELAUNCHED=1 {escaped_binary}\n");

    assert_eq!(logged_args, expected_cmd);

    Ok(())
}

#[test]
fn relaunch_returns_error_when_tmux_detach_command_fails() -> anyhow::Result<()> {
    let test_binary = env::current_exe()?;
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mock-tmux");
    let path = env::join_paths(once(fixture_dir).chain(env::split_paths(&env::var_os("PATH").unwrap_or_default())))?;

    let output = Command::new(test_binary)
        .args(["--exact", "relaunch_helper", "--ignored"])
        .env(HELPER_MODE_ENV, "relaunch_failure")
        .env("TMUX", "/tmp/tmux-1000/default,1234,0")
        .env_remove("TUIMUX_RELAUNCHED")
        .env("PATH", path)
        .env("MOCK_TMUX_FAIL", "1")
        .output()?;

    assert!(
        output.status.success(),
        "helper failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    Ok(())
}

#[test]
#[ignore = "run in an isolated subprocess helper"]
fn relaunch_helper() -> anyhow::Result<()> {
    let mode = env::var(HELPER_MODE_ENV)?;

    match mode.as_str() {
        "noop_no_tmux" => {
            let relaunched = maybe_relaunch_outside_tmux()?;
            assert!(!relaunched, "expected false when TMUX is not set");
        }
        "noop_already_relaunched" => {
            let relaunched = maybe_relaunch_outside_tmux()?;
            assert!(!relaunched, "expected false when TUIMUX_RELAUNCHED is set");
        }
        "relaunch_success" => {
            let relaunched = maybe_relaunch_outside_tmux()?;
            assert!(relaunched, "expected true when relaunch succeeds");
        }
        "relaunch_failure" => match maybe_relaunch_outside_tmux() {
            Ok(_) => anyhow::bail!("expected error when tmux detach fails"),
            Err(err) => {
                let err_msg = err.to_string();
                assert!(
                    err_msg.contains("mock detach failure"),
                    "expected error message to contain 'mock detach failure', got: {err_msg}"
                );
            }
        },
        _ => anyhow::bail!("unknown helper mode: {mode}"),
    }

    Ok(())
}
