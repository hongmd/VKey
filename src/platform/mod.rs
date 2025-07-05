// Platform-specific functionality
// This module contains platform-specific implementations
// for Vietnamese input method integration

use core_graphics::event::{CGEventTapProxy};
use std::collections::HashMap;
use once_cell::sync::OnceCell;
use bitflags::bitflags;
use rdev::{Keyboard, KeyboardState};
use log::debug;
use std::sync::Mutex;

// Platform type definitions
pub type CallbackFn = Box<dyn Fn(CGEventTapProxy, EventTapType, Option<PressedKey>, KeyModifier) -> bool>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventTapType {
    KeyDown,
    FlagsChanged,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressedKey {
    Char(char),
    Raw(u16),
}

bitflags! {
    pub struct KeyModifier: u32 {
        const MODIFIER_NONE     = 0b00000000;
        const MODIFIER_SHIFT    = 0b00000001;
        const MODIFIER_SUPER    = 0b00000010;
        const MODIFIER_CONTROL  = 0b00000100;
        const MODIFIER_ALT      = 0b00001000;
        const MODIFIER_CAPSLOCK = 0b00010000;
    }
}

impl KeyModifier {
    pub fn new() -> Self {
        Self::MODIFIER_NONE
    }

    pub fn add_shift(&mut self) {
        self.insert(Self::MODIFIER_SHIFT);
    }

    pub fn add_super(&mut self) {
        self.insert(Self::MODIFIER_SUPER);
    }

    pub fn add_control(&mut self) {
        self.insert(Self::MODIFIER_CONTROL);
    }

    pub fn add_alt(&mut self) {
        self.insert(Self::MODIFIER_ALT);
    }

    pub fn add_capslock(&mut self) {
        self.insert(Self::MODIFIER_CAPSLOCK);
    }

    pub fn is_shift(&self) -> bool {
        self.contains(Self::MODIFIER_SHIFT)
    }

    pub fn is_super(&self) -> bool {
        self.contains(Self::MODIFIER_SUPER)
    }

    pub fn is_control(&self) -> bool {
        self.contains(Self::MODIFIER_CONTROL)
    }

    pub fn is_alt(&self) -> bool {
        self.contains(Self::MODIFIER_ALT)
    }

    pub fn is_capslock(&self) -> bool {
        self.contains(Self::MODIFIER_CAPSLOCK)
    }
}

// Key constants
pub const KEY_ENTER: char = '\r';
pub const KEY_SPACE: char = ' ';
pub const KEY_TAB: char = '\t';
pub const KEY_DELETE: char = '\u{0008}'; // Backspace
pub const KEY_ESCAPE: char = '\u{001B}';

// Predefined character set for keyboard layout detection
pub const PREDEFINED_CHARS: [char; 47] = [
    'a', '`', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', 'q', 'w', 'e', 'r', 't',
    'y', 'u', 'i', 'o', 'p', '[', ']', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';', '\'', '\\',
    'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/',
];

// Keyboard layout character mapping
pub static KEYBOARD_LAYOUT_CHARACTER_MAP: OnceCell<Mutex<HashMap<char, char>>> = OnceCell::new();

/// Convert character to rdev::Key
pub fn get_key_from_char(c: char) -> rdev::Key {
    use rdev::Key::*;
    match &c {
        'a' => KeyA,
        '`' => BackQuote,
        '1' => Num1,
        '2' => Num2,
        '3' => Num3,
        '4' => Num4,
        '5' => Num5,
        '6' => Num6,
        '7' => Num7,
        '8' => Num8,
        '9' => Num9,
        '0' => Num0,
        '-' => Minus,
        '=' => Equal,
        'q' => KeyQ,
        'w' => KeyW,
        'e' => KeyE,
        'r' => KeyR,
        't' => KeyT,
        'y' => KeyY,
        'u' => KeyU,
        'i' => KeyI,
        'o' => KeyO,
        'p' => KeyP,
        '[' => LeftBracket,
        ']' => RightBracket,
        's' => KeyS,
        'd' => KeyD,
        'f' => KeyF,
        'g' => KeyG,
        'h' => KeyH,
        'j' => KeyJ,
        'k' => KeyK,
        'l' => KeyL,
        ';' => SemiColon,
        '\'' => Quote,
        '\\' => BackSlash,
        'z' => KeyZ,
        'x' => KeyX,
        'c' => KeyC,
        'v' => KeyV,
        'b' => KeyB,
        'n' => KeyN,
        'm' => KeyM,
        ',' => Comma,
        '.' => Dot,
        '/' => Slash,
        _ => Unknown(0),
    }
}

/// Build keyboard layout map using rdev
fn build_keyboard_layout_map(map: &mut HashMap<char, char>) {
    map.clear();
    if let Some(mut kb) = Keyboard::new() {
        for c in PREDEFINED_CHARS {
            let key = rdev::EventType::KeyPress(get_key_from_char(c));
            if let Some(s) = kb.add(&key) {
                if let Some(ch) = s.chars().last() {
                    map.insert(c, ch);
                }
            }
        }
        debug!("Built keyboard layout map with {} entries", map.len());
    } else {
        debug!("Failed to create rdev::Keyboard, falling back to static mapping");
        // Fallback to static mapping if rdev fails
        for c in PREDEFINED_CHARS {
            map.insert(c, c);
        }
    }
}

/// Initialize keyboard layout using rdev
pub fn initialize_keyboard_layout() {
    let mut map = HashMap::new();
    build_keyboard_layout_map(&mut map);
    if let Err(_) = KEYBOARD_LAYOUT_CHARACTER_MAP.set(Mutex::new(map)) {
        debug!("Keyboard layout map already initialized");
    } else {
        debug!("Keyboard layout map initialized successfully");
    }
}

/// Rebuild keyboard layout map when layout changes
pub fn rebuild_keyboard_layout_map() {
    // Get mutable reference to existing map if it exists
    if let Some(mutex) = KEYBOARD_LAYOUT_CHARACTER_MAP.get() {
        if let Ok(mut map) = mutex.lock() {
            debug!("Rebuilding keyboard layout map...");
            build_keyboard_layout_map(&mut map);
            debug!("Keyboard layout map rebuilt");
        } else {
            debug!("Failed to lock keyboard layout map mutex");
        }
    } else {
        debug!("Creating new keyboard layout map...");
        initialize_keyboard_layout();
        debug!("New keyboard layout map created");
    }
}

// MacOS keyboard handler
#[cfg(target_os = "macos")]
pub struct MacOSKeyboardHandler {
    input_type: crate::core::InputType,
    enabled: bool,
    current_buffer: String,
    system_integrated: bool,
}

#[cfg(target_os = "macos")]
impl MacOSKeyboardHandler {
    pub fn new(input_type: crate::core::InputType) -> Self {
        Self {
            input_type,
            enabled: false,
            current_buffer: String::new(),
            system_integrated: false,
        }
    }
    
    pub fn new_with_system_integration(input_type: crate::core::InputType) -> Result<Self, String> {
        let mut handler = Self::new(input_type);
        handler.system_integrated = true;
        Ok(handler)
    }
    
    pub fn set_input_type(&mut self, input_type: crate::core::InputType) {
        self.input_type = input_type;
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    pub fn get_current_buffer(&self) -> String {
        self.current_buffer.clone()
    }
    
    pub fn clear_buffer(&mut self) {
        self.current_buffer.clear();
    }
}

// System integration module
#[cfg(target_os = "macos")]
pub mod system_integration {
    use super::macos;
    
    pub fn has_accessibility_permissions() -> bool {
        macos::is_process_trusted()
    }
    
    pub fn request_accessibility_permissions() -> Result<(), String> {
        if macos::ensure_accessibility_permission() {
            Ok(())
        } else {
            Err("Failed to request accessibility permissions".to_string())
        }
    }
    
    pub fn remove_keyboard_hook() -> Result<(), String> {
        // This is a placeholder implementation
        // In a real implementation, you would clean up event taps and other resources
        Ok(())
    }
}

#[cfg(target_os = "macos")]
pub mod macos;

// Real CGEventTap implementation with proper dependencies
#[cfg(target_os = "macos")]
pub mod macos_ext;

#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(target_os = "macos")]
pub use macos_ext::{SystemTray, SystemTrayMenuItemKey}; 