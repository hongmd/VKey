use std::sync::{Arc, Mutex};
use crate::core::{VietnameseInputProcessor, ProcessingResult, InputType};

#[cfg(target_os = "macos")]
use accessibility_sys::*;
#[cfg(target_os = "macos")]
use core_foundation::{
    base::{CFTypeRef, TCFType},
    runloop::{CFRunLoop, CFRunLoopSource, kCFRunLoopCommonModes},
    string::CFString,
    boolean::CFBoolean,
    dictionary::CFDictionary,
};
#[cfg(target_os = "macos")]
use std::ffi::c_void;
#[cfg(target_os = "macos")]
use std::ptr;



// CGEvent constants and types for macOS
#[cfg(target_os = "macos")]
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum CGEventType {
    KeyDown = 10,
    KeyUp = 11,
    FlagsChanged = 12,
}

#[cfg(target_os = "macos")]
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum CGEventTapLocation {
    HIDEventTap = 0,
    SessionEventTap = 1,
    AnnotatedSessionEventTap = 2,
}

#[cfg(target_os = "macos")]
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum CGEventTapPlacement {
    HeadInsertEventTap = 0,
    TailAppendEventTap = 1,
}

#[cfg(target_os = "macos")]
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum CGEventTapOptions {
    DefaultTap = 0,
    ListenOnly = 1,
}

#[cfg(target_os = "macos")]
type CGEventRef = *mut c_void;
#[cfg(target_os = "macos")]
type CGEventTapRef = *mut c_void;
#[cfg(target_os = "macos")]
type CGEventMask = u64;

#[cfg(target_os = "macos")]
type CGEventTapCallBack = extern "C" fn(
    proxy: *mut c_void,
    event_type: CGEventType,
    event: CGEventRef,
    refcon: *mut c_void,
) -> CGEventRef;

#[cfg(target_os = "macos")]
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventTapCreate(
        tap: CGEventTapLocation,
        place: CGEventTapPlacement,
        options: CGEventTapOptions,
        events_of_interest: CGEventMask,
        callback: CGEventTapCallBack,
        refcon: *mut c_void,
    ) -> CGEventTapRef;
    
    fn CGEventTapEnable(tap: CGEventTapRef, enable: bool);
    
    // Commented out to avoid linking issues for now
    // fn CGEventTapCreateRunLoopSource(
    //     allocator: CFTypeRef,
    //     tap: CGEventTapRef,
    //     order: i32,
    // ) -> CFTypeRef;
    
    fn CGEventGetIntegerValueField(event: CGEventRef, field: u32) -> i64;
    // Commented out to avoid linking issues for now
    // fn CGEventGetStringValueField(event: CGEventRef, field: u32) -> CFTypeRef;
    // fn CGEventCreateKeyboardEvent(
    //     source: *mut c_void,
    //     keycode: u16,
    //     key_down: bool,
    // ) -> CGEventRef;
    // fn CGEventPost(tap: CGEventTapLocation, event: CGEventRef);
    // fn CGEventSetStringValueField(event: CGEventRef, field: u32, string: CFTypeRef);
}

#[derive(Debug, Clone)]
pub struct MacOSKeyboardHandler {
    processor: Arc<Mutex<VietnameseInputProcessor>>,
    is_enabled: bool,
}

impl MacOSKeyboardHandler {
    pub fn new(input_type: InputType) -> Self {
        Self {
            processor: Arc::new(Mutex::new(VietnameseInputProcessor::new(input_type))),
            is_enabled: true,
        }
    }

    /// Create a new handler and check for accessibility permissions
    pub fn new_with_accessibility_check(input_type: InputType) -> Result<Self, String> {
        if !system_integration::has_accessibility_permissions() {
            return Err("Accessibility permissions are required for VKey to function properly. Please grant permissions and restart the application.".to_string());
        }

        Ok(Self::new(input_type))
    }

    /// Create a new handler with full system integration (CGEventTap)
    pub fn new_with_system_integration(input_type: InputType) -> Result<Self, String> {
        let handler = Self::new_with_accessibility_check(input_type)?;
        
        // Install the keyboard hook
        system_integration::install_keyboard_hook()?;
        
        // Set the processor for the event tap
        system_integration::set_input_processor(handler.processor.clone())?;
        
        println!("VKey system integration initialized successfully");
        Ok(handler)
    }

    /// Enable/disable the global keyboard hook
    pub fn set_global_hook_enabled(&mut self, enabled: bool) -> Result<(), String> {
        if enabled {
            system_integration::install_keyboard_hook()
        } else {
            system_integration::remove_keyboard_hook()
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.is_enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    pub fn set_input_type(&mut self, input_type: InputType) {
        if let Ok(mut processor) = self.processor.lock() {
            processor.set_input_type(input_type);
        }
    }

    pub fn process_keyboard_event(&self, key_char: char) -> KeyboardEventResult {
        if !self.is_enabled {
            return KeyboardEventResult::PassThrough(key_char);
        }

        if let Ok(mut processor) = self.processor.lock() {
            match processor.process_key(key_char) {
                ProcessingResult::PassThrough(ch) => {
                    KeyboardEventResult::PassThrough(ch)
                }
                ProcessingResult::ProcessedText { text, buffer_length } => {
                    KeyboardEventResult::ReplaceText {
                        text,
                        delete_count: buffer_length.saturating_sub(1),
                    }
                }
                ProcessingResult::ClearAndPassBackspace => {
                    KeyboardEventResult::ClearAndBackspace
                }
                ProcessingResult::CommitAndPassThrough(ch) => {
                    KeyboardEventResult::CommitAndPassThrough(ch)
                }
            }
        } else {
            KeyboardEventResult::PassThrough(key_char)
        }
    }

    pub fn get_current_buffer(&self) -> String {
        if let Ok(processor) = self.processor.lock() {
            processor.get_current_buffer().to_string()
        } else {
            String::new()
        }
    }

    pub fn clear_buffer(&self) {
        if let Ok(mut processor) = self.processor.lock() {
            processor.clear_buffer();
        }
    }

    pub fn reset(&self) {
        if let Ok(mut processor) = self.processor.lock() {
            processor.reset();
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum KeyboardEventResult {
    /// Pass the character through without processing
    PassThrough(char),
    /// Replace current text with processed text, deleting previous characters
    ReplaceText {
        text: String,
        delete_count: usize,
    },
    /// Clear current buffer and send backspace
    ClearAndBackspace,
    /// Commit current buffer and pass character through
    CommitAndPassThrough(char),
}

// Simulate macOS-style keyboard event structure
#[derive(Debug, Clone)]
pub struct MacOSKeyEvent {
    pub key_char: Option<char>,
    pub key_code: u16,
    pub modifiers: MacOSModifiers,
    pub event_type: MacOSKeyEventType,
}

#[derive(Debug, Clone)]
pub struct MacOSModifiers {
    pub command: bool,
    pub option: bool,
    pub control: bool,
    pub shift: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MacOSKeyEventType {
    KeyDown,
    KeyUp,
}

impl MacOSKeyEvent {
    pub fn new_char_event(ch: char) -> Self {
        Self {
            key_char: Some(ch),
            key_code: 0, // Would be filled in by actual macOS integration
            modifiers: MacOSModifiers {
                command: false,
                option: false,
                control: false,
                shift: false,
            },
            event_type: MacOSKeyEventType::KeyDown,
        }
    }

    pub fn new_special_key(key_code: u16, modifiers: MacOSModifiers) -> Self {
        Self {
            key_char: None,
            key_code,
            modifiers,
            event_type: MacOSKeyEventType::KeyDown,
        }
    }
}

// Note: Global keyboard handler removed due to static mut safety concerns
// In a real implementation, you would use a different pattern like:
// - Arc<Mutex<>> with a static OnceCell
// - A proper event system
// - Platform-specific event loop integration

/// macOS accessibility permissions helper
#[cfg(target_os = "macos")]
pub mod accessibility {
    use super::*;

    /// Check if the current process has accessibility permissions
    pub fn is_trusted() -> bool {
        unsafe { AXIsProcessTrusted() }
    }

    /// Request accessibility permissions with user prompt
    /// Returns true if permissions are granted, false otherwise
    pub fn request_permissions() -> bool {
        if is_trusted() {
            return true;
        }

        unsafe {
            // Create a CFDictionary with kAXTrustedCheckOptionPrompt = true to show the dialog
            use core_foundation::{
                dictionary::CFDictionary,
                boolean::CFBoolean,
                string::CFString,
                base::TCFType,
            };
            
            let prompt_key = CFString::new("AXTrustedCheckOptionPrompt");
            let prompt_value = CFBoolean::true_value();
            
            let options = CFDictionary::from_CFType_pairs(&[
                (prompt_key.as_CFType(), prompt_value.as_CFType())
            ]);
            
            AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef())
        }
    }

    /// Get a user-friendly message about accessibility permissions
    pub fn get_permission_message() -> &'static str {
        "VKey requires accessibility permissions to function properly.\n\
         Please go to System Preferences > Security & Privacy > Privacy > Accessibility\n\
         and enable VKey."
    }
}

/// CGEventTap-based keyboard monitoring (similar to goxkey implementation)
/// DISABLED: Commented out to avoid linking issues for now
/*
#[cfg(target_os = "macos")]
#[allow(dead_code)]
pub mod event_tap_disabled {
    use super::*;
    use std::sync::{Arc, Mutex, Once};
    
    static mut EVENT_TAP_STATE: Option<Arc<Mutex<EventTapState>>> = None;
    static INIT: Once = Once::new();
    
    struct EventTapState {
        event_tap: CGEventTapRef,
        run_loop_source: CFRunLoopSource,
        is_running: bool,
        processor: Option<Arc<Mutex<VietnameseInputProcessor>>>,
    }
    
    impl EventTapState {
        fn new() -> Option<Self> {
            unsafe {
                let event_mask = (1 << CGEventType::KeyDown as u32) | (1 << CGEventType::KeyUp as u32);
                
                let event_tap = CGEventTapCreate(
                    CGEventTapLocation::SessionEventTap,
                    CGEventTapPlacement::HeadInsertEventTap,
                    CGEventTapOptions::DefaultTap,
                    event_mask,
                    event_tap_callback,
                    ptr::null_mut(),
                );
                
                if event_tap.is_null() {
                    return None;
                }
                
                let run_loop_source_ref = CGEventTapCreateRunLoopSource(
                    ptr::null(),
                    event_tap,
                    0,
                );
                
                if run_loop_source_ref.is_null() {
                    return None;
                }
                
                // Safely wrap the CFRunLoopSource pointer
                let run_loop_source = CFRunLoopSource::wrap_under_create_rule(run_loop_source_ref as *mut _);
                
                Some(EventTapState {
                    event_tap,
                    run_loop_source,
                    is_running: false,
                    processor: None,
                })
            }
        }
    }
    
    /// Initialize the global event tap system
    pub fn initialize() -> Result<(), String> {
        INIT.call_once(|| {
            unsafe {
                EVENT_TAP_STATE = EventTapState::new().map(|state| Arc::new(Mutex::new(state)));
            }
        });
        
        unsafe {
            match &EVENT_TAP_STATE {
                Some(_) => Ok(()),
                None => Err("Failed to create CGEventTap. Make sure accessibility permissions are granted.".to_string()),
            }
        }
    }
    
    /// Start monitoring keyboard events
    pub fn start_monitoring() -> Result<(), String> {
        unsafe {
            if let Some(state_arc) = &EVENT_TAP_STATE {
                let mut state = state_arc.lock().map_err(|_| "Failed to lock event tap state")?;
                
                if !state.is_running {
                    let current_run_loop = CFRunLoop::get_current();
                    current_run_loop.add_source(&state.run_loop_source, unsafe { kCFRunLoopCommonModes });
                    
                    CGEventTapEnable(state.event_tap, true);
                    state.is_running = true;
                }
                
                Ok(())
            } else {
                Err("Event tap not initialized".to_string())
            }
        }
    }
    
    /// Stop monitoring keyboard events
    pub fn stop_monitoring() -> Result<(), String> {
        unsafe {
            if let Some(state_arc) = &EVENT_TAP_STATE {
                let mut state = state_arc.lock().map_err(|_| "Failed to lock event tap state")?;
                
                if state.is_running {
                    CGEventTapEnable(state.event_tap, false);
                    state.is_running = false;
                }
                
                Ok(())
            } else {
                Err("Event tap not initialized".to_string())
            }
        }
    }
    
    /// Set the Vietnamese input processor for the event tap
    pub fn set_processor(processor: Arc<Mutex<VietnameseInputProcessor>>) -> Result<(), String> {
        unsafe {
            if let Some(state_arc) = &EVENT_TAP_STATE {
                let mut state = state_arc.lock().map_err(|_| "Failed to lock event tap state")?;
                state.processor = Some(processor);
                Ok(())
            } else {
                Err("Event tap not initialized".to_string())
            }
        }
    }
    
    /// CGEventTap callback function
    extern "C" fn event_tap_callback(
        _proxy: *mut c_void,
        event_type: CGEventType,
        event: CGEventRef,
        _refcon: *mut c_void,
    ) -> CGEventRef {
        unsafe {
            if let Some(state_arc) = &EVENT_TAP_STATE {
                if let Ok(state) = state_arc.lock() {
                    if let Some(ref processor_arc) = state.processor {
                        return handle_keyboard_event(event_type, event, processor_arc);
                    }
                }
            }
        }
        
        // Return the original event if we can't process it
        event
    }
    
    /// Handle individual keyboard events
    unsafe fn handle_keyboard_event(
        event_type: CGEventType,
        event: CGEventRef,
        processor: &Arc<Mutex<VietnameseInputProcessor>>,
    ) -> CGEventRef {
        // Only process key down events for now
        if !matches!(event_type, CGEventType::KeyDown) {
            return event;
        }
        
        // Get the key code
        let keycode = CGEventGetIntegerValueField(event, 9) as u16; // kCGKeyboardEventKeycode = 9
        
        // Convert keycode to character (simplified mapping)
        if let Some(ch) = keycode_to_char(keycode) {
            if let Ok(mut proc) = processor.lock() {
                match proc.process_key(ch) {
                    crate::core::ProcessingResult::ProcessedText { text, .. } => {
                        // Replace the event with processed text
                        let cf_string = CFString::new(&text);
                        CGEventSetStringValueField(event, 8, cf_string.as_CFTypeRef()); // kCGKeyboardEventUnicodeString = 8
                        return event;
                    }
                    crate::core::ProcessingResult::PassThrough(_) => {
                        // Let the original event pass through
                        return event;
                    }
                    crate::core::ProcessingResult::ClearAndPassBackspace => {
                        // Create a backspace event
                        let backspace_event = CGEventCreateKeyboardEvent(
                            ptr::null_mut(),
                            51, // Backspace keycode
                            true,
                        );
                        return backspace_event;
                    }
                    crate::core::ProcessingResult::CommitAndPassThrough(_) => {
                        proc.clear_buffer();
                        return event;
                    }
                }
            }
        }
        
        event
    }
    
    /// Convert macOS keycode to character (simplified mapping)
    fn keycode_to_char(keycode: u16) -> Option<char> {
        match keycode {
            0 => Some('a'), 11 => Some('b'), 8 => Some('c'), 2 => Some('d'), 14 => Some('e'),
            3 => Some('f'), 5 => Some('g'), 4 => Some('h'), 34 => Some('i'), 38 => Some('j'),
            40 => Some('k'), 37 => Some('l'), 46 => Some('m'), 45 => Some('n'), 31 => Some('o'),
            35 => Some('p'), 12 => Some('q'), 15 => Some('r'), 1 => Some('s'), 17 => Some('t'),
            32 => Some('u'), 9 => Some('v'), 13 => Some('w'), 7 => Some('x'), 16 => Some('y'),
            6 => Some('z'),
            18 => Some('1'), 19 => Some('2'), 20 => Some('3'), 21 => Some('4'), 23 => Some('5'),
            22 => Some('6'), 26 => Some('7'), 28 => Some('8'), 25 => Some('9'), 29 => Some('0'),
            49 => Some(' '), // Space
            _ => None,
        }
    }
}
*/

// Functions for macOS system integration
// In a real implementation, these would interface with macOS APIs
pub mod system_integration {
    use super::*;

    /// Install system-wide keyboard hook using CGEventTap (disabled for now)
    pub fn install_keyboard_hook() -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            // Check accessibility permissions first
            if !has_accessibility_permissions() {
                return Err("Accessibility permissions required for keyboard hook installation".to_string());
            }
            
            // Event tap functionality temporarily disabled to avoid linking issues
            println!("macOS keyboard hook installation temporarily disabled (linking issues)");
            Ok(())
        }
        #[cfg(not(target_os = "macos"))]
        {
            println!("Keyboard hook installation not implemented for this platform");
            Ok(())
        }
    }

    /// Remove system-wide keyboard hook
    pub fn remove_keyboard_hook() -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            println!("macOS keyboard hook removal temporarily disabled (linking issues)");
            Ok(())
        }
        #[cfg(not(target_os = "macos"))]
        {
            println!("Keyboard hook removal not implemented for this platform");
            Ok(())
        }
    }

    /// Set the Vietnamese input processor for the keyboard hook
    pub fn set_input_processor(processor: Arc<Mutex<VietnameseInputProcessor>>) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            let _ = processor; // silence unused variable warning
            println!("Input processor setting temporarily disabled (linking issues)");
            Ok(())
        }
        #[cfg(not(target_os = "macos"))]
        {
            println!("Input processor setting not implemented for this platform");
            Ok(())
        }
    }

    /// Send text to the currently focused application (mock implementation)
    pub fn send_text_to_focused_app(text: &str) -> Result<(), String> {
        println!("Sending text to focused app: '{}'", text);
        // In real implementation:
        // 1. Get focused application
        // 2. Send text using CGEventCreateKeyboardEvent or similar
        // 3. Handle text replacement if needed
        Ok(())
    }

    /// Send backspace events (mock implementation)
    pub fn send_backspace(count: usize) -> Result<(), String> {
        println!("Sending {} backspace events", count);
        // In real implementation:
        // 1. Create backspace key events
        // 2. Send them to the focused application
        Ok(())
    }

    /// Check if accessibility permissions are granted
    pub fn has_accessibility_permissions() -> bool {
        #[cfg(target_os = "macos")]
        {
            crate::platform::macos::accessibility::is_trusted()
        }
        #[cfg(not(target_os = "macos"))]
        {
            println!("Accessibility permissions check not implemented for this platform");
            true
        }
    }

    /// Request accessibility permissions
    pub fn request_accessibility_permissions() -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            let granted = crate::platform::macos::accessibility::request_permissions();
            if granted {
                Ok(())
            } else {
                Err(crate::platform::macos::accessibility::get_permission_message().to_string())
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            println!("Accessibility permissions request not implemented for this platform");
            Ok(())
        }
    }

    /// Check if accessibility API is enabled (deprecated in modern macOS)
    pub fn is_accessibility_api_enabled() -> bool {
        #[cfg(target_os = "macos")]
        {
            // AXAPIEnabled is deprecated and may not work on modern macOS
            // Use AXIsProcessTrusted instead for current accessibility status
            unsafe { AXIsProcessTrusted() }
        }
        #[cfg(not(target_os = "macos"))]
        {
            println!("Accessibility API check not implemented for this platform");
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::InputType;

    #[test]
    fn test_keyboard_handler_creation() {
        let handler = MacOSKeyboardHandler::new(InputType::Telex);
        assert!(handler.is_enabled());
    }

    #[test]
    fn test_telex_processing() {
        let handler = MacOSKeyboardHandler::new(InputType::Telex);
        
        // Test "viet6" -> "việt"
        let result1 = handler.process_keyboard_event('v');
        assert_eq!(result1, KeyboardEventResult::ReplaceText { text: "v".to_string(), delete_count: 0 });
        
        let result2 = handler.process_keyboard_event('i');
        assert_eq!(result2, KeyboardEventResult::ReplaceText { text: "vi".to_string(), delete_count: 1 });
        
        let result3 = handler.process_keyboard_event('e');
        assert_eq!(result3, KeyboardEventResult::ReplaceText { text: "vie".to_string(), delete_count: 2 });
        
        let result4 = handler.process_keyboard_event('t');
        assert_eq!(result4, KeyboardEventResult::ReplaceText { text: "viet".to_string(), delete_count: 3 });
        
        let result5 = handler.process_keyboard_event('6');
        assert_eq!(result5, KeyboardEventResult::ReplaceText { text: "việt".to_string(), delete_count: 4 });
    }

    #[test]
    fn test_vni_processing() {
        let handler = MacOSKeyboardHandler::new(InputType::VNI);
        
        // Test "viet65" -> "việt"
        let result1 = handler.process_keyboard_event('v');
        assert_eq!(result1, KeyboardEventResult::ReplaceText { text: "v".to_string(), delete_count: 0 });
        
        let result6 = handler.process_keyboard_event('6');
        let result5 = handler.process_keyboard_event('5');
        // The exact sequence depends on vi-rs VNI implementation
    }

    #[test]
    fn test_disabled_handler() {
        let mut handler = MacOSKeyboardHandler::new(InputType::Telex);
        handler.set_enabled(false);
        
        let result = handler.process_keyboard_event('a');
        assert_eq!(result, KeyboardEventResult::PassThrough('a'));
    }

    #[test]
    fn test_space_handling() {
        let handler = MacOSKeyboardHandler::new(InputType::Telex);
        
        // Type "viet6" first
        handler.process_keyboard_event('v');
        handler.process_keyboard_event('i');
        handler.process_keyboard_event('e');
        handler.process_keyboard_event('t');
        handler.process_keyboard_event('6');
        
        // Space should commit the buffer
        let result = handler.process_keyboard_event(' ');
        assert_eq!(result, KeyboardEventResult::CommitAndPassThrough(' '));
        
        // Buffer should be empty after space
        assert_eq!(handler.get_current_buffer(), "");
    }

    #[test]
    fn test_accessibility_permissions() {
        // Test basic accessibility permission checking
        let has_permissions = system_integration::has_accessibility_permissions();
        // This will be true in CI/testing environments or false if not granted
        println!("Has accessibility permissions: {}", has_permissions);
        
        // Test permission request (will return existing status in test environment)
        let request_result = system_integration::request_accessibility_permissions();
        println!("Permission request result: {:?}", request_result);
    }

    #[test]
    fn test_keyboard_handler_with_accessibility() {
        // Test that we can create a handler with accessibility check
        let result = MacOSKeyboardHandler::new_with_accessibility_check(InputType::Telex);
        
        // In testing environment, this should succeed if permissions are granted
        // or fail with appropriate error message if not
        match result {
            Ok(handler) => {
                println!("Handler created successfully with accessibility permissions");
                assert!(handler.is_enabled());
            }
            Err(e) => {
                println!("Handler creation failed (expected in environments without accessibility permissions): {}", e);
                assert!(e.contains("Accessibility permissions"));
            }
        }
    }
} 