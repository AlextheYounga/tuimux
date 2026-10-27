use std::env;
use std::ffi::OsString;
use std::fs;
use std::iter::once;
use std::path::{Path, PathBuf};
use std::process::Command;

#[test]
fn relaunch_invokes_tmux_detach_with_escaped_current_exe_path() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let test_binary = copy_tuimux_to_path_requiring_shell_escape(temp_dir.path())?;
    let log_file = temp_dir.path().join("mock_tmux.log");

    let output = Command::new(&test_binary)
        .env("TMUX", "/tmp/tmux-1000/default,1234,0")
        .env_remove("TUIMUX_RELAUNCHED")
        .env("PATH", path_with_mock_tmux()?)
        .env("MOCK_TMUX_LOG", &log_file)
        .env_remove("MOCK_TMUX_FAIL")
        .output()?;

    assert!(
        output.status.success(),
        "tuimux failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let escaped_binary = shell_escape::escape(test_binary.to_string_lossy());
    let expected_args = format!("detach-client\n-E\nTUIMUX_RELAUNCHED=1 {escaped_binary}\n");

    assert_eq!(fs::read_to_string(log_file)?, expected_args);

    Ok(())
}

#[test]
fn relaunch_exits_with_error_when_tmux_detach_command_fails() -> anyhow::Result<()> {
    let output = Command::new(env!("CARGO_BIN_EXE_tuimux"))
        .env("TMUX", "/tmp/tmux-1000/default,1234,0")
        .env_remove("TUIMUX_RELAUNCHED")
        .env("PATH", path_with_mock_tmux()?)
        .env("MOCK_TMUX_FAIL", "1")
        .env_remove("MOCK_TMUX_LOG")
        .output()?;

    assert!(!output.status.success(), "expected tuimux to fail when tmux detach fails");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("mock detach failure"),
        "expected error message to contain 'mock detach failure', got: {stderr}"
    );

    Ok(())
}

fn copy_tuimux_to_path_requiring_shell_escape(temp_dir: &Path) -> anyhow::Result<PathBuf> {
    let binary_dir = temp_dir.join("directory with spaces");
    fs::create_dir(&binary_dir)?;

    let test_binary = binary_dir.join("tuimux's binary");
    fs::copy(env!("CARGO_BIN_EXE_tuimux"), &test_binary)?;

    Ok(test_binary)
}

fn path_with_mock_tmux() -> anyhow::Result<OsString> {
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mock-tmux");
    let current_path = env::var_os("PATH").unwrap_or_default();

    Ok(env::join_paths(once(fixture_dir).chain(env::split_paths(&current_path)))?)
}
