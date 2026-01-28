use crate::models::config::Config;

#[derive(Debug)]
#[allow(dead_code)]
pub enum ParseError {
    InvalidToml(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidToml(msg) => write!(f, "Invalid TOML: {msg}"),
        }
    }
}

impl std::error::Error for ParseError {}

/// Parses TOML configuration content into a Config struct.
///
/// # Errors
///
/// Returns `ParseError::InvalidToml` if the TOML content is malformed.
pub fn parse_config(content: &str) -> Result<Config, ParseError> {
    toml::from_str(content).map_err(|e| ParseError::InvalidToml(e.to_string()))
}

/// Serializes a Config struct into pretty-printed TOML.
///
/// # Errors
///
/// Returns `ParseError::InvalidToml` if serialization fails.
pub fn serialize_config(config: &Config) -> Result<String, ParseError> {
    toml::to_string_pretty(config).map_err(|e| ParseError::InvalidToml(e.to_string()))
}
