// Platform-specific functionality
// This module contains platform-specific implementations
// for Vietnamese input method integration

use core_graphics::event::{CGEventTapProxy};
use std::collections::HashMap;
use once_cell::sync::OnceCell;
use bitflags::bitflags;

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
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

// Keyboard layout character mapping
pub static KEYBOARD_LAYOUT_CHARACTER_MAP: OnceCell<HashMap<char, char>> = OnceCell::new();

pub fn initialize_keyboard_layout() {
    let mut map = HashMap::new();
    
    // Standard QWERTY layout mapping
    map.insert('a', 'a');
    map.insert('s', 's');
    map.insert('d', 'd');
    map.insert('f', 'f');
    map.insert('g', 'g');
    map.insert('h', 'h');
    map.insert('j', 'j');
    map.insert('k', 'k');
    map.insert('l', 'l');
    map.insert('z', 'z');
    map.insert('x', 'x');
    map.insert('c', 'c');
    map.insert('v', 'v');
    map.insert('b', 'b');
    map.insert('n', 'n');
    map.insert('m', 'm');
    map.insert('q', 'q');
    map.insert('w', 'w');
    map.insert('e', 'e');
    map.insert('r', 'r');
    map.insert('t', 't');
    map.insert('y', 'y');
    map.insert('u', 'u');
    map.insert('i', 'i');
    map.insert('o', 'o');
    map.insert('p', 'p');
    
    // Numbers
    map.insert('1', '1');
    map.insert('2', '2');
    map.insert('3', '3');
    map.insert('4', '4');
    map.insert('5', '5');
    map.insert('6', '6');
    map.insert('7', '7');
    map.insert('8', '8');
    map.insert('9', '9');
    map.insert('0', '0');
    
    // Symbols
    map.insert('-', '-');
    map.insert('=', '=');
    map.insert('[', '[');
    map.insert(']', ']');
    map.insert('\\', '\\');
    map.insert(';', ';');
    map.insert('\'', '\'');
    map.insert(',', ',');
    map.insert('.', '.');
    map.insert('/', '/');
    
    KEYBOARD_LAYOUT_CHARACTER_MAP.set(map).unwrap();
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