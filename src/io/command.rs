use std::process::Command;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub enum CommandError {
    ExecutionFailed(String),
    NotFound(String),
}

/// Runs `checkupdates` to scan for official package updates with retry logic.
///
/// Retries up to 3 times with 2 second delays on failure.
///
/// # Errors
///
/// Returns `CommandError::ExecutionFailed` if the command fails to execute
/// or returns a non-zero exit status after all retries (except exit code 2, which means no updates).
pub fn run_checkupdates() -> Result<String, CommandError> {
    const MAX_RETRIES: u32 = 3;
    const RETRY_DELAY_SECS: u64 = 2;

    let mut last_error = String::new();

    for attempt in 1..=MAX_RETRIES {
        let output = Command::new("checkupdates")
            .output()
            .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;

        // Exit code 2 means no updates available (not an error)
        if output.status.success() || output.status.code() == Some(2) {
            return Ok(String::from_utf8_lossy(&output.stdout).to_string());
        }

        last_error = String::from_utf8_lossy(&output.stderr).to_string();

        if attempt < MAX_RETRIES {
            eprintln!(
                "checkupdates failed (attempt {attempt}/{MAX_RETRIES}), retrying in {RETRY_DELAY_SECS}s..."
            );
            thread::sleep(Duration::from_secs(RETRY_DELAY_SECS));
        }
    }

    Err(CommandError::ExecutionFailed(format!(
        "Failed after {MAX_RETRIES} attempts: {last_error}"
    )))
}

/// Runs `paru -Qua` to query AUR package updates.
///
/// # Errors
///
/// Returns `CommandError::ExecutionFailed` if the command fails to execute
/// or returns a non-zero exit status.
pub fn run_paru_query_aur() -> Result<String, CommandError> {
    let output = Command::new("paru")
        .args(["-Qua"])
        .output()
        .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;

    if !output.status.success() {
        return Err(CommandError::ExecutionFailed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Checks if a command exists in PATH using `which`.
#[must_use]
pub fn check_command_exists(command: &str) -> bool {
    Command::new("which")
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
