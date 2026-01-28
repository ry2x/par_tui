use std::process::Command;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub enum CommandError {
    ExecutionFailed(String),
    #[allow(dead_code)]
    NotFound(String),
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExecutionFailed(msg) => write!(f, "Command execution failed: {msg}"),
            Self::NotFound(msg) => write!(f, "Command not found: {msg}"),
        }
    }
}

impl std::error::Error for CommandError {}

/// Runs `checkupdates` to scan for official package updates with retry logic.
///
/// Retries up to 3 times with 2 second delays on failure.
/// Calls the provided callback with retry progress messages.
///
/// # Errors
///
/// Returns `CommandError::ExecutionFailed` if the command fails to execute
/// or returns a non-zero exit status after all retries (except exit code 2, which means no updates).
pub fn run_checkupdates_with_callback<F>(mut on_retry: F) -> Result<String, CommandError>
where
    F: FnMut(u32, u32),
{
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
            on_retry(attempt, MAX_RETRIES);
            thread::sleep(Duration::from_secs(RETRY_DELAY_SECS));
        }
    }

    Err(CommandError::ExecutionFailed(format!(
        "Failed after {MAX_RETRIES} attempts: {last_error}"
    )))
}

/// Runs `checkupdates` to scan for official package updates with retry logic.
///
/// Retries up to 3 times with 2 second delays on failure.
///
/// # Errors
///
/// Returns `CommandError::ExecutionFailed` if the command fails to execute
/// or returns a non-zero exit status after all retries (except exit code 2, which means no updates).
#[allow(dead_code)]
pub fn run_checkupdates() -> Result<String, CommandError> {
    run_checkupdates_with_callback(|_, _| {})
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
