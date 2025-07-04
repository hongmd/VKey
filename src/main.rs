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

/// Restore the original word by sending backspaces and the original text
fn do_restore_word(handle: Handle) {
    if let Ok(processor) = INPUT_PROCESSOR.lock() {
        let buffer = processor.get_current_buffer();
        if !buffer.is_empty() {
            // Send backspaces to clear the processed text
            let _ = send_backspace(handle, buffer.chars().count());
            // Send the original buffer back
            let _ = send_string(handle, buffer);
        }
    }
}

/// Transform keys based on Vietnamese input rules
fn transform_key(handle: Handle, key: PressedKey, modifiers: KeyModifier) -> bool {
    eprintln!("Vietnamese enabled: {}", VIETNAMESE_ENABLED.load(Ordering::Relaxed));
    if !VIETNAMESE_ENABLED.load(Ordering::Relaxed) {
        return false; // Don't process if Vietnamese input is disabled
    }

    if let PressedKey::Char(mut character) = key {
        // Apply case conversion based on Shift key
        if modifiers.is_shift() && character.is_ascii_lowercase() {
            character = character.to_ascii_uppercase();
        }
        
        if let Ok(mut processor) = INPUT_PROCESSOR.lock() {
            match processor.process_key(character) {
                ProcessingResult::ProcessedText { text, buffer_length } => {
                    // Clear previous text and send new text
                    if buffer_length > 0 {
                        let _ = send_backspace(handle, buffer_length);
                    }
                    let _ = send_string(handle, &text);
                    return true; // Block original key
                }
                ProcessingResult::PassThrough(_) => {
                    // Let the original character through
                    return false;
                }
                ProcessingResult::ClearAndPassBackspace => {
                    // Clear buffer and let backspace through
                    return false;
                }
                ProcessingResult::CommitAndPassThrough(_) => {
                    // Commit current buffer and let character through
                    return false;
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

        // Handle special keys when Vietnamese is enabled
        if VIETNAMESE_ENABLED.load(Ordering::Relaxed) {
            if let PressedKey::Char(ch) = key {
                match ch {
                    KEY_ESCAPE => {
                        do_restore_word(handle);
                        return true;
                    }
                    KEY_TAB | KEY_ENTER => {
                        if let Ok(mut processor) = INPUT_PROCESSOR.lock() {
                            processor.clear_buffer();
                        }
                        return false; // Let these keys through
                    }
                    _ => {}
                }
            }
        }

        // Transform regular characters through Vietnamese input method
        return transform_key(handle, key, modifiers);
    }

    false
}
