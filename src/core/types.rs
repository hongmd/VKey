use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents the input method for Vietnamese text
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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