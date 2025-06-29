use serde::{Deserialize, Serialize};
use crate::core::types::{InputType, Encoding, InputMode, KeyboardConfig, AdvancedSettings};
use crate::error::Result;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub input_type: InputType,
    pub encoding: Encoding,
    pub input_mode: InputMode,
    pub keyboard: KeyboardConfig,
    pub advanced: AdvancedSettings,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            input_type: InputType::Telex,
            encoding: Encoding::Unicode,
            input_mode: InputMode::English,
            keyboard: KeyboardConfig::default(),
            advanced: AdvancedSettings::default(),
        }
    }
}

impl AppConfig {
    /// Load configuration from a file
    pub fn load(path: &str) -> Result<Self> {
        let config_str = std::fs::read_to_string(path)
            .map_err(|e| crate::error::VKeyError::ConfigError(e.to_string()))?;
        
        serde_json::from_str(&config_str)
            .map_err(|e| crate::error::VKeyError::ConfigError(e.to_string()))
    }

    /// Save configuration to a file
    pub fn save(&self, path: &str) -> Result<()> {
        let config_str = serde_json::to_string_pretty(self)
            .map_err(|e| crate::error::VKeyError::ConfigError(e.to_string()))?;
        
        std::fs::write(path, config_str)
            .map_err(|e| crate::error::VKeyError::ConfigError(e.to_string()))
    }
} 