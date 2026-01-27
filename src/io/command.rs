use std::process::Command;

#[derive(Debug)]
pub enum CommandError {
    ExecutionFailed(String),
    NotFound(String),
}

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

pub fn check_command_exists(command: &str) -> bool {
    Command::new("which")
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
