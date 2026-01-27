use crate::models::config::Config;

#[derive(Debug)]
pub enum ParseError {
    InvalidToml(String),
}

pub fn parse_config(content: &str) -> Result<Config, ParseError> {
    toml::from_str(content).map_err(|e| ParseError::InvalidToml(e.to_string()))
}

pub fn serialize_config(config: &Config) -> Result<String, ParseError> {
    toml::to_string_pretty(config).map_err(|e| ParseError::InvalidToml(e.to_string()))
}
