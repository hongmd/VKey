use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents the input method for Vietnamese text
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputType {
    /// Telex input method (e.g., aa -> â)
    Telex,
    /// VNI input method (e.g., a6 -> â)
    VNI,
    /// VIQR input method (e.g., a^ -> â)
    VIQR,
}

impl fmt::Display for InputType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputType::Telex => write!(f, "Telex"),
            InputType::VNI => write!(f, "VNI"),
            InputType::VIQR => write!(f, "VIQR"),
        }
    }
}

/// Represents the character encoding for Vietnamese text
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Encoding {
    /// Unicode UTF-8 encoding
    Unicode,
    /// TCVN3 (ABC) encoding
    TCVN3,
    /// VNI-Win encoding
    VNIWin,
}

impl fmt::Display for Encoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Encoding::Unicode => write!(f, "Unicode"),
            Encoding::TCVN3 => write!(f, "TCVN3"),
            Encoding::VNIWin => write!(f, "VNI-Win"),
        }
    }
}

/// Represents the current input mode
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputMode {
    /// Vietnamese input mode
    Vietnamese,
    /// English input mode
    English,
}

impl fmt::Display for InputMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputMode::Vietnamese => write!(f, "Tiếng Việt"),
            InputMode::English => write!(f, "English"),
        }
    }
}

/// Configuration for keyboard modifiers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyboardConfig {
    pub shift_enabled: bool,
    pub ctrl_enabled: bool,
    pub cmd_enabled: bool,
    pub home_enabled: bool,
    pub beep_enabled: bool,
}

impl Default for KeyboardConfig {
    fn default() -> Self {
        Self {
            shift_enabled: false,
            ctrl_enabled: false,
            cmd_enabled: true,
            home_enabled: true,
            beep_enabled: false,
        }
    }
}

/// Additional configuration options for the VKey UI
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdvancedSettings {
    /// Replace oa, uy with oa, uy
    pub replace_oa_uy: bool,
    /// Spell checking
    pub spell_check: bool,
    /// Auto restart on typos
    pub auto_restart_typos: bool,
    /// Write Vietnamese with capital letters
    pub vietnamese_capital: bool,
    /// Smart input mode switching
    pub smart_switching: bool,
    /// Remember encoding by app
    pub remember_encoding: bool,
    /// Allow z w j f as silent consonants
    pub allow_silent_consonants: bool,
    /// Auto-correct spelling mistakes
    pub auto_correct_spelling: bool,
    /// Temporarily disable spell check
    pub temp_disable_spell_check: bool,
    /// Temporarily disable VKey
    pub temp_disable_openkey: bool,
}

impl Default for AdvancedSettings {
    fn default() -> Self {
        Self {
            replace_oa_uy: false,
            spell_check: true,
            auto_restart_typos: false,
            vietnamese_capital: false,
            smart_switching: true,
            remember_encoding: true,
            allow_silent_consonants: false,
            auto_correct_spelling: false,
            temp_disable_spell_check: false,
            temp_disable_openkey: false,
        }
    }
} 