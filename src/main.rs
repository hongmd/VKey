use gpui::{
    App, AppContext, Application, Bounds, WindowBounds, WindowOptions, px, size
};

mod core;
mod error;
mod platform;
mod ui;
use std::thread;

use ui::VKeyApp;
use core::AppConfig;

#[cfg(target_os = "macos")]
use platform::system_integration;
use platform::{
    run_event_listener, send_backspace, send_string, CallbackFn, EventTapType, Handle, KeyModifier, PressedKey, KEY_ENTER, KEY_ESCAPE,
    KEY_TAB,
};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::sync::mpsc::{self, Sender};
use once_cell::sync::Lazy;
use crate::core::{VietnameseInputProcessor, ProcessingResult};

// Global state for Vietnamese input processing
static VIETNAMESE_ENABLED: AtomicBool = AtomicBool::new(true); // Start with Vietnamese enabled by default
static INPUT_PROCESSOR: Lazy<Mutex<VietnameseInputProcessor>> = Lazy::new(|| {
    // Load config to get initial input type
    let config = AppConfig::load_default().unwrap_or_default();
    
    // Always start with Vietnamese enabled by default
    VIETNAMESE_ENABLED.store(true, Ordering::Relaxed);
    
    Mutex::new(VietnameseInputProcessor::new(config.input_type))
});

// Global configuration
static GLOBAL_CONFIG: Lazy<Mutex<AppConfig>> = Lazy::new(|| {
    Mutex::new(AppConfig::load_default().unwrap_or_default())
});

// Global hotkey state
static mut HOTKEY_MODIFIERS: KeyModifier = KeyModifier::MODIFIER_NONE;
static HOTKEY_MATCHING: AtomicBool = AtomicBool::new(false);

// Raw key constants
const RAW_KEY_GLOBE: u16 = 179; // Globe key on Mac keyboards

// System tray event types
#[derive(Debug, Clone)]
pub enum SystemTrayEvent {
    ShowUI,
    ToggleVietnamese,
    SetInputTypeTelex,
    SetInputTypeVNI,
}

// Global system tray event channel
static SYSTEM_TRAY_SENDER: Lazy<Mutex<Option<Sender<SystemTrayEvent>>>> = Lazy::new(|| {
    Mutex::new(None)
});

// Function to send system tray events
pub fn send_system_tray_event(event: SystemTrayEvent) {
    if let Ok(sender_guard) = SYSTEM_TRAY_SENDER.lock() {
        if let Some(ref sender) = *sender_guard {
            if let Err(e) = sender.send(event) {
                eprintln!("Failed to send system tray event: {}", e);
            }
        }
    }
}

fn main() {
    eprintln!("Starting VKey application...");
    
    // Initialize platform-specific components
    #[cfg(target_os = "macos")]
    platform::initialize_keyboard_layout();
    
    let result = std::panic::catch_unwind(|| {
        // Check and request permissions before starting the application
        #[cfg(target_os = "macos")]
        {
            eprintln!("Checking accessibility permissions...");
            if !system_integration::has_accessibility_permissions() {
                eprintln!("Accessibility permissions not granted. Requesting permissions...");
                match system_integration::request_accessibility_permissions() {
                    Ok(_) => {
                        if system_integration::has_accessibility_permissions() {
                            eprintln!("Accessibility permissions granted successfully!");
                        } else {
                            eprintln!("Accessibility permissions were not granted. The app will work with limited functionality.");
                            eprintln!("To enable full functionality, please:");
                            eprintln!("1. Go to System Preferences > Security & Privacy > Privacy > Accessibility");
                            eprintln!("2. Enable VKey in the list");
                            eprintln!("3. Restart the application");
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to request accessibility permissions: {}", e);
                        eprintln!("The app will work with limited functionality.");
                    }
                }
            } else {
                eprintln!("Accessibility permissions already granted!");
            }
            
            // Note: Keyboard hook will be installed by VKeyApp during initialization
        }

        Application::new().run(|cx: &mut App| {
            gpui_component::init(cx);

            eprintln!("Creating window...");
            
            // Set up system tray event channel
            let (sender, receiver) = mpsc::channel::<SystemTrayEvent>();
            if let Ok(mut sender_guard) = SYSTEM_TRAY_SENDER.lock() {
                *sender_guard = Some(sender);
            }
            
            let bounds = Bounds::centered(None, size(px(650.), px(560.)), cx);
            match cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_, cx| {
                    eprintln!("Initializing VKeyApp...");
                    cx.new(|_| {
                        let mut app = VKeyApp::new_with_system_tray_receiver(Some(receiver));
                        
                        // Mark permissions as checked since we did it in main
                        app.set_permissions_checked(true);
                        
                        // Initialize the system tray
                        match app.initialize_system_tray() {
                            Ok(_) => eprintln!("VKeyApp system tray initialized successfully"),
                            Err(e) => eprintln!("Failed to initialize VKeyApp system tray: {}", e),
                        }

                        // Initialize the keyboard system integration
                        match app.initialize_keyboard_system() {
                            Ok(_) => {
                                thread::spawn(|| {
                                    let handler = Box::new(event_handler) as CallbackFn;
                                    run_event_listener(&handler);
                                });
                                eprintln!("VKeyApp keyboard system initialized successfully");
                            }
                            Err(e) => {
                                eprintln!("Failed to initialize VKeyApp keyboard system: {}", e);
                            }
                        }
                        
                        eprintln!("VKeyApp initialized successfully");
                        app
                    })
                },
            ) {
                Ok(_) => eprintln!("Window created successfully"),
                Err(e) => eprintln!("Failed to create window: {:?}", e),
            }
        });
    });
    
    match result {
        Ok(_) => {
            eprintln!("Application finished normally");
            
            // Clean up the keyboard hook when the application exits
            #[cfg(target_os = "macos")]
            {
                eprintln!("Cleaning up keyboard hook...");
                if let Err(e) = system_integration::remove_keyboard_hook() {
                    eprintln!("Failed to remove keyboard hook: {}", e);
                } else {
                    eprintln!("Keyboard hook removed successfully");
                }
            }
        }
        Err(e) => eprintln!("Application panicked: {:?}", e),
    }
}

/// Toggle Vietnamese input mode with config sync
fn toggle_vietnamese() {
    let current = VIETNAMESE_ENABLED.load(Ordering::Relaxed);
    VIETNAMESE_ENABLED.store(!current, Ordering::Relaxed);
    
    // Update global config
    if let Ok(mut config) = GLOBAL_CONFIG.lock() {
        let _ = config.set_vietnamese_mode(!current);
    }
    
    if let Ok(mut processor) = INPUT_PROCESSOR.lock() {
        processor.clear_buffer();
    }
    
    eprintln!("Vietnamese input: {}", if !current { "enabled" } else { "disabled" });
}

/// Check if the current key combination matches the configured hotkey
fn is_hotkey_match(modifiers: KeyModifier, key: Option<PressedKey>) -> bool {
    if let Ok(config) = GLOBAL_CONFIG.lock() {
        if let Some(ref hotkey) = config.global_hotkey {
            // Parse hotkey string and compare
            // For now, default to cmd+space
            if hotkey.contains("cmd") && hotkey.contains("space") {
                return modifiers.is_super() && key.map_or(false, |k| match k {
                    PressedKey::Char(ch) => ch == ' ',
                    _ => false,
                });
            }
        }
    }
    
    // Default hotkey: cmd+space
    modifiers.is_super() && key.map_or(false, |k| match k {
        PressedKey::Char(ch) => ch == ' ',
        _ => false,
    })
}

/// Handle backspace using goxkey-style approach
/// This implements the "backspace technique" used by Vietnamese input methods
fn handle_backspace_goxkey_style(handle: Handle) -> bool {
    eprintln!("Handling backspace with goxkey-style approach");
    
    // First check if text is selected in the application
    #[cfg(target_os = "macos")]
    let has_text_selection = {
        use platform::is_in_text_selection;
        is_in_text_selection()
    };
    #[cfg(not(target_os = "macos"))]
    let has_text_selection = false;
    
    if has_text_selection {
        eprintln!("Text selection detected - clearing buffer and letting backspace pass through");
        // Clear our internal buffer since the user is deleting selected text
        if let Ok(mut processor) = INPUT_PROCESSOR.lock() {
            processor.clear_buffer();
        }
        // Let the system handle the deletion of selected text
        return false;
    }
    
    // If Vietnamese input is not enabled, let backspace pass through normally
    if !VIETNAMESE_ENABLED.load(Ordering::Relaxed) {
        eprintln!("Vietnamese not enabled - letting backspace pass through");
        return false;
    }
    
    // Handle Vietnamese input backspace
    if let Ok(mut processor) = INPUT_PROCESSOR.lock() {
        let buffer_before = processor.get_current_buffer().to_string();
        eprintln!("Current buffer before backspace: '{}'", buffer_before);
        
        // Process backspace through Vietnamese processor
        match processor.handle_backspace() {
            ProcessingResult::ProcessedText { text, buffer_length } => {
                eprintln!("Backspace processed - clearing {} chars, sending: '{}'", buffer_length, text);
                // Use goxkey-style backspace technique:
                // 1. Send backspaces to clear the previously displayed text
                if buffer_length > 0 {
                    let _ = send_backspace(handle, buffer_length);
                }
                // 2. Send the new transformed text
                if !text.is_empty() {
                    let _ = send_string(handle, &text);
                }
                return true; // Block the original backspace
            }
            ProcessingResult::ClearAndPassBackspace => {
                eprintln!("Buffer cleared - letting backspace pass through");
                // Buffer is now empty, let the backspace pass through to delete 
                // the character before our Vietnamese input started
                return false;
            }
            ProcessingResult::PassThrough(_) => {
                eprintln!("Backspace passed through");
                // Let backspace pass through
                return false;
            }
            ProcessingResult::RestoreText { text, buffer_length } => {
                eprintln!("Restoring text: '{}', clearing {} chars", text, buffer_length);
                // Clear the current displayed text and send the original text
                if buffer_length > 0 {
                    let _ = send_backspace(handle, buffer_length);
                }
                if !text.is_empty() {
                    let _ = send_string(handle, &text);
                }
                return true;
            }
        }
    }
    
    // Fallback: let backspace pass through
    eprintln!("Fallback - letting backspace pass through");
    false
}

/// Restore the original word by sending backspaces and the original text
fn do_restore_word(handle: Handle) {
    if let Ok(processor) = INPUT_PROCESSOR.lock() {
        let original_text = processor.get_restore_text();
        let display_length = processor.get_display_buffer().chars().count();
        
        if !original_text.is_empty() {
            eprintln!("Restoring word: '{}', clearing {} chars", original_text, display_length);
            // Send backspaces to clear the processed text
            if display_length > 0 {
                let _ = send_backspace(handle, display_length);
            }
            // Send the original buffer back
            let _ = send_string(handle, &original_text);
        }
    }
}

/// Transform keys based on Vietnamese input rules with improved goxkey-style handling
fn transform_key(handle: Handle, key: PressedKey, modifiers: KeyModifier) -> bool {
    eprintln!("Vietnamese enabled: {}", VIETNAMESE_ENABLED.load(Ordering::Relaxed));
    
    if let PressedKey::Char(character) = key {
        // Handle backspace with goxkey-style approach
        if character == '\u{8}' { // Backspace
            return handle_backspace_goxkey_style(handle);
        }
        
        // Handle special shifted character transformations (always apply, regardless of Vietnamese mode)
        let mut transformed_character = character;
        if modifiers.is_shift() {
            transformed_character = match character {
                // Handle Shift+. => >
                '.' => '>',
                // Handle other shifted characters
                ',' => '<',
                ';' => ':',
                '\'' => '"',
                '[' => '{',
                ']' => '}',
                '\\' => '|',
                '/' => '?',
                '1' => '!',
                '2' => '@',
                '3' => '#',
                '4' => '$',
                '5' => '%',
                '6' => '^',
                '7' => '&',
                '8' => '*',
                '9' => '(',
                '0' => ')',
                '-' => '_',
                '=' => '+',
                '`' => '~',
                // Apply case conversion for letters
                c if c.is_ascii_lowercase() => c.to_ascii_uppercase(),
                // Keep other characters as is
                c => c,
            };
        }
        
        // If the character was transformed and Vietnamese is not enabled, send the transformed character
        if transformed_character != character && !VIETNAMESE_ENABLED.load(Ordering::Relaxed) {
            let _ = send_string(handle, &transformed_character.to_string());
            return true; // Block original key and send transformed character
        }
        
        // If Vietnamese is not enabled, let the original character through
        if !VIETNAMESE_ENABLED.load(Ordering::Relaxed) {
            return false;
        }
        
        // Before processing Vietnamese input, check if there's text selection
        // If there is, we should clear our buffer and let the character replace the selection
        #[cfg(target_os = "macos")]
        {
            use platform::is_in_text_selection;
            if is_in_text_selection() {
                eprintln!("Text selection detected for character input - clearing buffer");
                if let Ok(mut processor) = INPUT_PROCESSOR.lock() {
                    processor.clear_buffer();
                }
                // Continue with Vietnamese processing but with cleared buffer
            }
        }
        
        // Vietnamese input processing
        if let Ok(mut processor) = INPUT_PROCESSOR.lock() {
            match processor.process_key(transformed_character) {
                ProcessingResult::ProcessedText { text, buffer_length } => {
                    // Use goxkey-style technique: clear previous text and send new text
                    eprintln!("Sending Vietnamese text: '{}', clearing {} chars", text, buffer_length);
                    if buffer_length > 0 {
                        let _ = send_backspace(handle, buffer_length);
                    }
                    let _ = send_string(handle, &text);
                    return true; // Block original key
                }
                ProcessingResult::PassThrough(_) => {
                    // Let the original character through
                    eprintln!("Vietnamese processor passed character through");
                    return false;
                }
                ProcessingResult::ClearAndPassBackspace => {
                    // Clear buffer and let backspace through
                    eprintln!("Vietnamese processor cleared buffer for backspace");
                    return false;
                }
                ProcessingResult::RestoreText { text, buffer_length } => {
                    // Restore original text (typically for Escape key)
                    eprintln!("Vietnamese processor restoring text: '{}', clearing {} chars", text, buffer_length);
                    if buffer_length > 0 {
                        let _ = send_backspace(handle, buffer_length);
                    }
                    if !text.is_empty() {
                        let _ = send_string(handle, &text);
                    }
                    return true;
                }
            }
        }
    }
    
    false
}

/// Main event handler for keyboard events
fn event_handler(
    handle: Handle,
    event_type: EventTapType,
    pressed_key: Option<PressedKey>,
    modifiers: KeyModifier,
) -> bool {
    eprintln!("Event received: type={:?}, key={:?}, modifiers={:?}", event_type, pressed_key, modifiers);

    unsafe {
        HOTKEY_MODIFIERS = modifiers;
    }

    // Handle hotkey combinations
    if event_type == EventTapType::FlagsChanged {
        let hotkey_active = unsafe { HOTKEY_MODIFIERS.is_super() };
        HOTKEY_MATCHING.store(hotkey_active, Ordering::Relaxed);
        return false; // Don't block modifier key events
    }

    // Check for toggle hotkey
    if let Some(key) = pressed_key {
        if is_hotkey_match(modifiers, Some(key)) {
            toggle_vietnamese();
            return true; // Block the hotkey from reaching other applications
        }

        // Handle Cmd key combinations - let them pass through
        if modifiers.is_super() {
            eprintln!("Cmd key combination detected, letting it pass through");
            // Clear Vietnamese buffer when Cmd is used (user is probably switching apps or using shortcuts)
            if let Ok(mut processor) = INPUT_PROCESSOR.lock() {
                processor.new_word();
            }
            return false; // Don't block Cmd key combinations
        }

        // Handle special keys when Vietnamese is enabled
        if VIETNAMESE_ENABLED.load(Ordering::Relaxed) {
            if let PressedKey::Char(ch) = key {
                match ch {
                    KEY_ESCAPE => {
                        // Escape key handling is now integrated into the Vietnamese processor
                        return transform_key(handle, key, modifiers);
                    }
                    KEY_TAB | KEY_ENTER => {
                        // Tab and Enter handling is now integrated into the Vietnamese processor
                        return transform_key(handle, key, modifiers);
                    }
                    '\u{8}' => { // Backspace
                        // Backspace handling is done in transform_key function
                        return transform_key(handle, key, modifiers);
                    }
                    _ => {
                        // Handle other modifier combinations that should reset the buffer
                        if modifiers.is_alt() || modifiers.is_control() {
                            if let Ok(mut processor) = INPUT_PROCESSOR.lock() {
                                processor.new_word();
                            }
                            return false; // Let these combinations pass through
                        }
                    }
                }
            }
        }

        // Handle raw key events (arrow keys, etc.)
        if let PressedKey::Raw(raw_keycode) = key {
            if raw_keycode == RAW_KEY_GLOBE {
                toggle_vietnamese();
                return true;
            }
            
            // Arrow keys should reset the Vietnamese buffer
            const RAW_ARROW_UP: u16 = 0x7e;
            const RAW_ARROW_DOWN: u16 = 0x7d;
            const RAW_ARROW_LEFT: u16 = 0x7b;
            const RAW_ARROW_RIGHT: u16 = 0x7c;
            
            if [RAW_ARROW_UP, RAW_ARROW_DOWN, RAW_ARROW_LEFT, RAW_ARROW_RIGHT].contains(&raw_keycode) {
                if let Ok(mut processor) = INPUT_PROCESSOR.lock() {
                    processor.new_word();
                }
                return false; // Let arrow keys pass through
            }
        }

        // Transform regular characters through Vietnamese input method
        return transform_key(handle, key, modifiers);
    }

    false
}
