use gpui::{
    App, AppContext, Application, Bounds, WindowBounds, WindowOptions, px, size
};

mod core;
mod error;
mod platform;
mod ui;
use std::thread;

use ui::VKeyApp;

#[cfg(target_os = "macos")]
use platform::system_integration;
use platform::{
    add_app_change_callback, ensure_accessibility_permission, run_event_listener, send_backspace,
    send_string, CallbackFn, EventTapType, Handle, KeyModifier, PressedKey, KEY_DELETE, KEY_ENTER, KEY_ESCAPE,
    KEY_SPACE, KEY_TAB,
};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::core::{VietnameseInputProcessor, InputType, ProcessingResult};

// Global state for Vietnamese input processing
static VIETNAMESE_ENABLED: AtomicBool = AtomicBool::new(true); // Start with Vietnamese enabled for testing
static INPUT_PROCESSOR: Lazy<Mutex<VietnameseInputProcessor>> = Lazy::new(|| {
    Mutex::new(VietnameseInputProcessor::new(InputType::Telex))
});

// Global hotkey state
static mut HOTKEY_MODIFIERS: KeyModifier = KeyModifier::MODIFIER_NONE;
static HOTKEY_MATCHING: AtomicBool = AtomicBool::new(false);

// Raw key constants
const RAW_KEY_GLOBE: u16 = 179; // Globe key on Mac keyboards

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
            eprintln!("Creating window...");
            let bounds = Bounds::centered(None, size(px(650.), px(560.)), cx);
            match cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_, cx| {
                    eprintln!("Initializing VKeyApp...");
                    cx.new(|_| {
                        let mut app = VKeyApp::new();
                        
                        // Mark permissions as checked since we did it in main
                        app.set_permissions_checked(true);
                        
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

/// Toggle Vietnamese input mode
fn toggle_vietnamese() {
    let current = VIETNAMESE_ENABLED.load(Ordering::Relaxed);
    VIETNAMESE_ENABLED.store(!current, Ordering::Relaxed);
    
    if let Ok(mut processor) = INPUT_PROCESSOR.lock() {
        processor.clear_buffer();
    }
    
    eprintln!("Vietnamese input: {}", if !current { "enabled" } else { "disabled" });
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
fn do_transform_keys(handle: Handle, _force: bool) -> bool {
    if let Ok(mut processor) = INPUT_PROCESSOR.lock() {
        let buffer = processor.get_current_buffer();
        if buffer.is_empty() {
            return false;
        }
        
        // Get the current transformed text
        let current_text = get_transformed_text(&processor);
        
        // If transformation occurred, replace the text
        if current_text != buffer {
            // Send backspaces to clear original text
            let _ = send_backspace(handle, buffer.chars().count());
            // Send transformed text
            let _ = send_string(handle, &current_text);
            return true;
        }
    }
    false
}

/// Get transformed text from the processor
fn get_transformed_text(processor: &VietnameseInputProcessor) -> String {
    let buffer = processor.get_current_buffer();
    if buffer.is_empty() {
        return String::new();
    }
    
    // For now, just return the buffer - the transformation happens in process_key
    // In a more complete implementation, this could apply additional transformations
    eprintln!("Getting transformed text from buffer: '{}'", buffer);
    buffer.to_string()
}

/// Handle macro replacement (placeholder)
fn do_macro_replace(_handle: Handle, _macro_target: &str) {
    // Placeholder for macro functionality
}

/// Handle the result of Vietnamese processing
fn handle_processing_result(handle: Handle, result: ProcessingResult) -> bool {
    match result {
        ProcessingResult::PassThrough(_) => {
            eprintln!("PassThrough - letting original key through");
            false // Let the original key through
        }
        ProcessingResult::ProcessedText { text, buffer_length } => {
            eprintln!("ProcessedText - replacing with: '{}'", text);
            // Send backspaces to clear the original buffer
            if buffer_length > 0 {
                let _ = send_backspace(handle, buffer_length);
            }
            // Send the processed text
            let _ = send_string(handle, &text);
            true // Suppress the original keystroke
        }
        ProcessingResult::ClearAndPassBackspace => {
            eprintln!("ClearAndPassBackspace - clearing and passing backspace");
            false // Let the original backspace through
        }
        ProcessingResult::CommitAndPassThrough(_) => {
            eprintln!("CommitAndPassThrough - committing and passing through");
            false // Let the original key through
        }
    }
}

fn event_handler(
    handle: Handle,
    event_type: EventTapType,
    pressed_key: Option<PressedKey>,
    modifiers: KeyModifier,
) -> bool {
    eprintln!("Event received: type={:?}, key={:?}, modifiers={:?}", event_type, pressed_key, modifiers);
    
    let pressed_key_code = pressed_key.and_then(|p| match p {
        PressedKey::Char(c) => Some(c),
        _ => None,
    });

    // Handle modifier key changes
    if event_type == EventTapType::FlagsChanged {
        unsafe {
            if modifiers.is_empty() {
                // Modifier keys are released
                if HOTKEY_MATCHING.load(Ordering::Relaxed) {
                    toggle_vietnamese();
                }
                HOTKEY_MODIFIERS = KeyModifier::MODIFIER_NONE;
                HOTKEY_MATCHING.store(false, Ordering::Relaxed);
            } else {
                HOTKEY_MODIFIERS = modifiers;
            }
        }
        return false;
    }

    // Check for hotkey combinations (simple Cmd+Space for now)
    let is_hotkey_matched = modifiers.is_super() && pressed_key_code == Some(KEY_SPACE);
    HOTKEY_MATCHING.store(is_hotkey_matched, Ordering::Relaxed);

    if let Some(pressed_key) = pressed_key {
        match pressed_key {
            PressedKey::Raw(raw_keycode) => {
                if raw_keycode == RAW_KEY_GLOBE {
                    toggle_vietnamese();
                    return true;
                }
            }
            PressedKey::Char(keycode) => {
                let vietnamese_enabled = VIETNAMESE_ENABLED.load(Ordering::Relaxed);
                
                if vietnamese_enabled {
                    match keycode {
                        KEY_SPACE => {
                            // Handle space properly - commit current buffer first
                            if let Ok(mut processor) = INPUT_PROCESSOR.lock() {
                                eprintln!("Processing space - buffer before: '{}'", processor.get_current_buffer());
                                let result = processor.handle_space();
                                eprintln!("Space result: {:?}", result);
                                
                                return handle_processing_result(handle, result);
                            }
                        }
                        KEY_ENTER | KEY_TAB | KEY_ESCAPE => {
                            // Commit current buffer for other commit keys
                            if let Ok(mut processor) = INPUT_PROCESSOR.lock() {
                                processor.clear_buffer();
                            }
                        }
                        KEY_DELETE => {
                            if modifiers.is_empty() || modifiers.is_shift() {
                                // Handle backspace in Vietnamese context
                                if let Ok(mut processor) = INPUT_PROCESSOR.lock() {
                                    if !processor.get_current_buffer().is_empty() {
                                        eprintln!("Processing backspace - buffer before: '{}'", processor.get_current_buffer());
                                        let result = processor.handle_backspace();
                                        eprintln!("Backspace result: {:?}", result);
                                        eprintln!("Buffer after backspace: '{}'", processor.get_current_buffer());
                                        
                                        return handle_processing_result(handle, result);
                                    }
                                }
                            }
                        }
                        c => {
                            // Handle regular character input
                            if c.is_ascii_alphabetic() && !modifiers.is_super() && !modifiers.is_alt() {
                                if let Ok(mut processor) = INPUT_PROCESSOR.lock() {
                                    let input_char = if modifiers.is_shift() || modifiers.is_capslock() {
                                        c.to_ascii_uppercase()
                                    } else {
                                        c
                                    };
                                    
                                    eprintln!("Processing key: '{}' (Vietnamese enabled: {})", input_char, vietnamese_enabled);
                                    let result = processor.process_key(input_char);
                                    eprintln!("Process result: {:?}", result);
                                    eprintln!("Current buffer: '{}'", processor.get_current_buffer());
                                    
                                    // Handle the processing result properly
                                    return handle_processing_result(handle, result);
                                }
                            } else {
                                eprintln!("Key '{}' not processed - modifiers: {:?}, is_alphabetic: {}", c, modifiers, c.is_ascii_alphabetic());
                            }
                        }
                    }
                }
            }
        }
    }
    
    false
}
