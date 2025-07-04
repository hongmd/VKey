use serde::{Deserialize, Serialize};
use crate::core::types::{InputType, Encoding, InputMode, KeyboardConfig, AdvancedSettings};
use crate::error::Result;
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub input_type: InputType,
    pub encoding: Encoding,
    pub input_mode: InputMode,
    pub keyboard: KeyboardConfig,
    pub advanced: AdvancedSettings,
    /// Global hotkey configuration for toggling Vietnamese input
    pub global_hotkey: Option<String>,
    /// Auto-save configuration on changes
    pub auto_save: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            input_type: InputType::Telex,
            encoding: Encoding::Unicode,
            input_mode: InputMode::English,
            keyboard: KeyboardConfig::default(),
            advanced: AdvancedSettings::default(),
            global_hotkey: Some("cmd+space".to_string()),
            auto_save: true,
        }
    }
}

impl AppConfig {
    /// Get the default configuration directory path
    pub fn get_config_dir() -> Result<PathBuf> {
        #[cfg(target_os = "macos")]
        {
            if let Some(home) = std::env::var_os("HOME") {
                let mut path = PathBuf::from(home);
                path.push("Library");
                path.push("Application Support");
                path.push("VKey");
                return Ok(path);
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            if let Some(appdata) = std::env::var_os("APPDATA") {
                let mut path = PathBuf::from(appdata);
                path.push("VKey");
                return Ok(path);
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            if let Some(home) = std::env::var_os("HOME") {
                let mut path = PathBuf::from(home);
                path.push(".config");
                path.push("vkey");
                return Ok(path);
            }
        }
        
        // Fallback to current directory
        Ok(PathBuf::from("."))
    }
    
    /// Get the default configuration file path
    pub fn get_config_path() -> Result<PathBuf> {
        let mut path = Self::get_config_dir()?;
        path.push("config.json");
        Ok(path)
    }
    
    /// Ensure the configuration directory exists
    pub fn ensure_config_dir() -> Result<PathBuf> {
        let config_dir = Self::get_config_dir()?;
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)
                .map_err(|e| crate::error::VKeyError::ConfigError(
                    format!("Failed to create config directory: {}", e)
                ))?;
        }
        Ok(config_dir)
    }
    
    /// Load configuration from the default location
    pub fn load_default() -> Result<Self> {
        Self::ensure_config_dir()?;
        let config_path = Self::get_config_path()?;
        
        if config_path.exists() {
            Self::load(config_path.to_str().unwrap_or("config.json"))
        } else {
            // Create default config if none exists
            let default_config = Self::default();
            default_config.save_default()?;
            Ok(default_config)
        }
    }
    
    /// Load configuration from a file
    pub fn load(path: &str) -> Result<Self> {
        let config_str = std::fs::read_to_string(path)
            .map_err(|e| crate::error::VKeyError::ConfigError(
                format!("Failed to read config file '{}': {}", path, e)
            ))?;
        
        let mut config: Self = serde_json::from_str(&config_str)
            .map_err(|e| crate::error::VKeyError::ConfigError(
                format!("Failed to parse config file '{}': {}", path, e)
            ))?;
        
        // Validate and fix any issues
        config.validate_and_fix()?;
        
        Ok(config)
    }
    
    /// Save configuration to the default location
    pub fn save_default(&self) -> Result<()> {
        Self::ensure_config_dir()?;
        let config_path = Self::get_config_path()?;
        self.save(config_path.to_str().unwrap_or("config.json"))
    }

    /// Save configuration to a file
    pub fn save(&self, path: &str) -> Result<()> {
        let config_str = serde_json::to_string_pretty(self)
            .map_err(|e| crate::error::VKeyError::ConfigError(
                format!("Failed to serialize config: {}", e)
            ))?;
        
        std::fs::write(path, config_str)
            .map_err(|e| crate::error::VKeyError::ConfigError(
                format!("Failed to write config file '{}': {}", path, e)
            ))
    }
    
    /// Toggle Vietnamese input mode
    pub fn toggle_vietnamese_mode(&mut self) -> Result<()> {
        self.input_mode = match self.input_mode {
            InputMode::Vietnamese => InputMode::English,
            InputMode::English => InputMode::Vietnamese,
        };
        
        if self.auto_save {
            self.save_default()?;
        }
        
        Ok(())
    }
    
    /// Set Vietnamese input mode
    pub fn set_vietnamese_mode(&mut self, enabled: bool) -> Result<()> {
        self.input_mode = if enabled {
            InputMode::Vietnamese
        } else {
            InputMode::English
        };
        
        if self.auto_save {
            self.save_default()?;
        }
        
        Ok(())
    }
    
    /// Check if Vietnamese input is currently enabled
    pub fn is_vietnamese_enabled(&self) -> bool {
        matches!(self.input_mode, InputMode::Vietnamese)
    }
    
    /// Update configuration and auto-save if enabled
    pub fn update_and_save(&mut self) -> Result<()> {
        if self.auto_save {
            self.save_default()?;
        }
        Ok(())
    }
    
    /// Validate configuration and fix common issues
    fn validate_and_fix(&mut self) -> Result<()> {
        // Ensure global hotkey is valid or reset to default
        if let Some(ref hotkey) = self.global_hotkey {
            if hotkey.trim().is_empty() || !self.is_valid_hotkey(hotkey) {
                eprintln!("Invalid global hotkey '{}', resetting to default", hotkey);
                self.global_hotkey = Some("cmd+space".to_string());
            }
        } else {
            self.global_hotkey = Some("cmd+space".to_string());
        }
        
        // Validate keyboard config
        self.validate_keyboard_config();
        
        // Validate advanced settings
        self.validate_advanced_settings();
        
        Ok(())
    }
    
    /// Check if a hotkey string is valid
    fn is_valid_hotkey(&self, hotkey: &str) -> bool {
        let parts: Vec<String> = hotkey.split('+').map(|s| s.trim().to_lowercase()).collect();
        
        if parts.is_empty() {
            return false;
        }
        
        // Check for valid modifier keys
        let mut has_modifier = false;
        let mut has_key = false;
        
        for part in &parts {
            match part.as_str() {
                "cmd" | "command" | "ctrl" | "control" | "alt" | "option" | "shift" => {
                    has_modifier = true;
                }
                "space" | "enter" | "tab" | "escape" | "backspace" => {
                    has_key = true;
                }
                key if key.len() == 1 && key.chars().next().unwrap().is_ascii_alphabetic() => {
                    has_key = true;
                }
                _ => {
                    // Unknown key
                    return false;
                }
            }
        }
        
        has_modifier && has_key
    }
    
    /// Validate and fix keyboard configuration
    fn validate_keyboard_config(&mut self) {
        // Ensure at least one modifier is enabled for keyboard shortcuts
        if !self.keyboard.shift_enabled 
            && !self.keyboard.ctrl_enabled 
            && !self.keyboard.cmd_enabled 
            && !self.keyboard.home_enabled {
            eprintln!("No keyboard modifiers enabled, enabling cmd key by default");
            self.keyboard.cmd_enabled = true;
        }
    }
    
    /// Validate and fix advanced settings
    fn validate_advanced_settings(&mut self) {
        // No specific validation needed for advanced settings currently
        // but this is where we'd add validation for features that depend on each other
    }
    
    /// Get available hotkey options for the UI
    pub fn get_hotkey_options() -> Vec<(&'static str, &'static str)> {
        vec![
            ("cmd+space", "⌘ + Space"),
            ("ctrl+space", "⌃ + Space"), 
            ("cmd+shift+v", "⌘ + ⇧ + V"),
            ("ctrl+shift+v", "⌃ + ⇧ + V"),
            ("cmd+i", "⌘ + I"),
            ("ctrl+i", "⌃ + I"),
        ]
    }
    
    /// Set a validated global hotkey
    pub fn set_global_hotkey(&mut self, hotkey: &str) -> Result<()> {
        if self.is_valid_hotkey(hotkey) {
            self.global_hotkey = Some(hotkey.to_string());
            if self.auto_save {
                self.save_default()?;
            }
            Ok(())
        } else {
            Err(crate::error::VKeyError::ConfigError(
                format!("Invalid hotkey format: '{}'", hotkey)
            ))
        }
    }
    
    /// Get a human-readable description of the current global hotkey
    pub fn get_hotkey_description(&self) -> String {
        if let Some(ref hotkey) = self.global_hotkey {
            Self::get_hotkey_options()
                .iter()
                .find(|(key, _)| *key == hotkey)
                .map(|(_, desc)| desc.to_string())
                .unwrap_or_else(|| hotkey.clone())
        } else {
            "None".to_string()
        }
    }
    
    /// Reset to default configuration
    pub fn reset_to_default(&mut self) -> Result<()> {
        *self = Self::default();
        if self.auto_save {
            self.save_default()?;
        }
        Ok(())
    }
} 