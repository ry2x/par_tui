use std::process::Command;

#[derive(Debug)]
pub enum CommandError {
    ExecutionFailed(String),
    NotFound(String),
}

/// Runs `checkupdates` to scan for official package updates.
///
/// # Errors
///
/// Returns `CommandError::ExecutionFailed` if the command fails to execute
/// or returns a non-zero exit status (except exit code 2, which means no updates).
pub fn run_checkupdates() -> Result<String, CommandError> {
    let output = Command::new("checkupdates")
        .output()
        .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;

    if !output.status.success() && output.status.code() != Some(2) {
        return Err(CommandError::ExecutionFailed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
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
