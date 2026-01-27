use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub exclude: ExcludeConfig,
    #[serde(default)]
    pub behavior: BehaviorConfig,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ExcludeConfig {
    #[serde(default)]
    pub permanent: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BehaviorConfig {
    #[serde(default = "default_warn_stale_system")]
    pub warn_stale_system: bool,
    #[serde(default)]
    pub extra_args: Vec<String>,
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            warn_stale_system: true,
            extra_args: Vec::new(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            exclude: ExcludeConfig::default(),
            behavior: BehaviorConfig::default(),
        }
    }
}

fn default_warn_stale_system() -> bool {
    true
}
