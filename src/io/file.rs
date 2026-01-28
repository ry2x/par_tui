use std::fs;
use std::path::Path;

#[derive(Debug)]
#[allow(dead_code)]
pub enum FileError {
    ReadFailed(String),
    WriteFailed(String),
    NotFound,
}

impl std::fmt::Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadFailed(msg) => write!(f, "Failed to read file: {msg}"),
            Self::WriteFailed(msg) => write!(f, "Failed to write file: {msg}"),
            Self::NotFound => write!(f, "File not found"),
        }
    }
}

impl std::error::Error for FileError {}

/// Reads configuration file from the given path.
///
/// # Errors
///
/// Returns `FileError::NotFound` if the file does not exist,
/// or `FileError::ReadFailed` if reading fails for other reasons.
pub fn read_config(path: &Path) -> Result<String, FileError> {
    fs::read_to_string(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            FileError::NotFound
        } else {
            FileError::ReadFailed(e.to_string())
        }
    })
}

/// Writes configuration content to the given path.
///
/// Creates parent directories if they don't exist.
///
/// # Errors
///
/// Returns `FileError::WriteFailed` if directory creation or file writing fails.
pub fn write_config(path: &Path, content: &str) -> Result<(), FileError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| FileError::WriteFailed(e.to_string()))?;
    }
    fs::write(path, content).map_err(|e| FileError::WriteFailed(e.to_string()))
}
