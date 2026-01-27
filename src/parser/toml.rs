use crate::models::config::Config;

#[derive(Debug)]
pub enum ParseError {
    InvalidToml(String),
}

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
