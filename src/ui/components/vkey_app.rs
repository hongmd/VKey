use gpui::{
    div, prelude::*, rgb, Context, IntoElement, Render, Styled, Window, MouseButton, Entity
};
use crate::core::{AppConfig, InputType, Encoding, InputMode, VietnameseInputProcessor};
use std::sync::mpsc::Receiver;

#[cfg(target_os = "macos")]
use crate::platform::{MacOSKeyboardHandler, system_integration, SystemTray};

// Add gpui-component imports using correct module paths
use gpui_component::{
    dropdown::{Dropdown, DropdownState, DropdownEvent},
};

pub struct VKeyApp {
    config: AppConfig,
    vietnamese_processor: VietnameseInputProcessor,
    input_text: String,
    #[cfg(target_os = "macos")]
    keyboard_handler: Option<MacOSKeyboardHandler>,
    #[cfg(target_os = "macos")]
    system_tray: Option<SystemTray>,
    system_tray_receiver: Option<Receiver<crate::SystemTrayEvent>>,
    permissions_checked: bool,
    // Dropdown states for proper selection tracking
    input_type_dropdown: Option<Entity<DropdownState<Vec<String>>>>,
    encoding_dropdown: Option<Entity<DropdownState<Vec<String>>>>,
}

impl VKeyApp {
    pub fn new() -> Self {
        Self::new_with_system_tray_receiver(None)
    }

    pub fn new_with_system_tray_receiver(receiver: Option<Receiver<crate::SystemTrayEvent>>) -> Self {
        // Load configuration from default location or create new one
        let config = AppConfig::load_default().unwrap_or_else(|e| {
            eprintln!("Failed to load config: {}. Using default.", e);
            AppConfig::default()
        });
        let vietnamese_processor = VietnameseInputProcessor::new(config.input_type);
        
        #[cfg(target_os = "macos")]
        let keyboard_handler = Some(MacOSKeyboardHandler::new(config.input_type));
        
        Self {
            config,
            vietnamese_processor,
            input_text: String::new(),
            #[cfg(target_os = "macos")]
            keyboard_handler,
            #[cfg(target_os = "macos")]
            system_tray: None,
            system_tray_receiver: receiver,
            permissions_checked: false,
            input_type_dropdown: None,
            encoding_dropdown: None,
        }
    }

    /// Initialize the system tray
    pub fn initialize_system_tray(&mut self) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            let system_tray = SystemTray::new();
            self.system_tray = Some(system_tray);
            self.setup_system_tray_callbacks()?;
            println!("System tray initialized successfully");
        }
        
        #[cfg(not(target_os = "macos"))]
        {
            println!("System tray: Platform not supported");
        }
        
        Ok(())
    }

    /// Initialize the keyboard system integration
    pub fn initialize_keyboard_system(&mut self) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            // Check accessibility permissions
            if !system_integration::has_accessibility_permissions() {
                return Err("Accessibility permissions are required but not granted".to_string());
            }
            
            // Initialize the keyboard handler with system integration
            match crate::platform::MacOSKeyboardHandler::new_with_system_integration(self.config.input_type) {
                Ok(handler) => {
                    self.keyboard_handler = Some(handler);
                    println!("Vietnamese input system initialized with full system integration");
                }
                Err(e) => {
                    // Fall back to basic handler without system integration
                    println!("System integration failed ({}), using basic handler", e);
                    self.keyboard_handler = Some(crate::platform::MacOSKeyboardHandler::new(self.config.input_type));
                }
            }
            
            // Set the initial input mode state
            match self.config.input_mode {
                crate::core::InputMode::Vietnamese => {
                    if let Some(ref mut handler) = self.keyboard_handler {
                        handler.set_enabled(true);
                    }
                }
                crate::core::InputMode::English => {
                    if let Some(ref mut handler) = self.keyboard_handler {
                        handler.set_enabled(false);
                    }
                }
            }
            
            println!("Vietnamese input system ready for macOS");
        }
        
        #[cfg(not(target_os = "macos"))]
        {
            println!("Vietnamese input system: Platform not supported");
        }
        
        Ok(())
    }
    

    /// Process a keyboard character through Vietnamese input
    pub fn process_vietnamese_input(&mut self, ch: char) -> String {
        use crate::core::ProcessingResult;
        
        match self.vietnamese_processor.process_key(ch) {
            ProcessingResult::ProcessedText { text, .. } => {
                self.input_text = text.clone();
                text
            }
            ProcessingResult::PassThrough(ch) => {
                self.input_text.push(ch);
                ch.to_string()
            }
            ProcessingResult::ClearAndPassBackspace => {
                self.input_text.clear();
                "\u{8}".to_string()
            }
            ProcessingResult::RestoreText { text, .. } => {
                self.input_text = text.clone();
                text
            }
        }
    }

    /// Toggle Vietnamese input on/off
    pub fn toggle_vietnamese_input(&mut self) {
        match self.config.toggle_vietnamese_mode() {
            Ok(_) => {
                #[cfg(target_os = "macos")]
                {
                    if let Some(ref mut handler) = self.keyboard_handler {
                        handler.set_enabled(self.config.is_vietnamese_enabled());
                    }
                    self.update_system_tray_state();
                    self.update_system_tray_title();
                }
                println!("Vietnamese input toggled to: {}", 
                    if self.config.is_vietnamese_enabled() { "ON" } else { "OFF" });
            }
            Err(e) => {
                eprintln!("Failed to toggle Vietnamese input: {}", e);
            }
        }
    }
    
    /// Set Vietnamese input mode explicitly
    pub fn set_vietnamese_input(&mut self, enabled: bool) {
        match self.config.set_vietnamese_mode(enabled) {
            Ok(_) => {
                #[cfg(target_os = "macos")]
                {
                    if let Some(ref mut handler) = self.keyboard_handler {
                        handler.set_enabled(enabled);
                    }
                    self.update_system_tray_state();
                    self.update_system_tray_title();
                }
                println!("Vietnamese input set to: {}", 
                    if enabled { "ON" } else { "OFF" });
            }
            Err(e) => {
                eprintln!("Failed to set Vietnamese input: {}", e);
            }
        }
    }

    /// Handle input type change
    pub fn set_input_type(&mut self, input_type: InputType) {
        self.config.input_type = input_type;
        self.vietnamese_processor.set_input_type(input_type);
        
        // Rebuild keyboard layout when input type changes
        crate::platform::rebuild_keyboard_layout_map();
        
        // Save configuration
        if let Err(e) = self.config.update_and_save() {
            eprintln!("Failed to save config after input type change: {}", e);
        }
        
        #[cfg(target_os = "macos")]
        {
            if let Some(ref mut handler) = self.keyboard_handler {
                handler.set_input_type(input_type);
            }
            self.update_system_tray_state();
            self.update_system_tray_title();
        }
    }
    
    /// Handle encoding change
    pub fn set_encoding(&mut self, encoding: Encoding) {
        self.config.encoding = encoding;
        
        // Save configuration
        if let Err(e) = self.config.update_and_save() {
            eprintln!("Failed to save config after encoding change: {}", e);
        }
    }
    
    /// Reset configuration to defaults
    pub fn reset_to_defaults(&mut self) {
        match self.config.reset_to_default() {
            Ok(_) => {
                // Update processor and handler with new settings
                self.vietnamese_processor.set_input_type(self.config.input_type);
                
                // Rebuild keyboard layout when configuration is reset
                crate::platform::rebuild_keyboard_layout_map();
                
                #[cfg(target_os = "macos")]
                {
                    if let Some(ref mut handler) = self.keyboard_handler {
                        handler.set_input_type(self.config.input_type);
                        handler.set_enabled(self.config.is_vietnamese_enabled());
                    }
                    self.update_system_tray_state();
                    self.update_system_tray_title();
                }
                
                println!("Configuration reset to defaults");
            }
            Err(e) => {
                eprintln!("Failed to reset configuration: {}", e);
            }
        }
    }

    /// Get current input buffer for display
    pub fn get_current_input_buffer(&self) -> String {
        #[cfg(target_os = "macos")]
        if let Some(ref handler) = self.keyboard_handler {
            return handler.get_current_buffer();
        }
        
        self.vietnamese_processor.get_current_buffer().to_string()
    }

    /// Get current display buffer for showing transformed text
    pub fn get_current_display_buffer(&self) -> String {
        #[cfg(target_os = "macos")]
        if let Some(ref handler) = self.keyboard_handler {
            // If handler has display buffer method, use it
            return handler.get_current_buffer();
        }
        
        self.vietnamese_processor.get_display_buffer().to_string()
    }

    /// Clear the input buffer
    pub fn clear_input_buffer(&mut self) {
        self.vietnamese_processor.clear_buffer();
        self.input_text.clear();
        
        #[cfg(target_os = "macos")]
        if let Some(ref mut handler) = self.keyboard_handler {
            handler.clear_buffer();
        }
    }

    /// Get the previous word for restoration
    pub fn get_previous_word(&self) -> String {
        self.vietnamese_processor.get_previous_word().to_string()
    }

    /// Check if the processor is currently tracking input
    pub fn is_tracking_input(&self) -> bool {
        self.vietnamese_processor.is_tracking()
    }

    /// Start a new word (reset buffers and enable tracking)
    pub fn start_new_word(&mut self) {
        self.vietnamese_processor.new_word();
        
        #[cfg(target_os = "macos")]
        if let Some(ref mut handler) = self.keyboard_handler {
            // If handler has new_word method, call it
            handler.clear_buffer();
        }
    }

    /// Check if accessibility permissions are granted
    pub fn has_accessibility_permissions(&self) -> bool {
        #[cfg(target_os = "macos")]
        {
            system_integration::has_accessibility_permissions()
        }
        #[cfg(not(target_os = "macos"))]
        {
            true // Non-macOS platforms don't need accessibility permissions
        }
    }

    /// Request accessibility permissions
    pub fn request_accessibility_permissions(&mut self) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            self.permissions_checked = true;
            system_integration::request_accessibility_permissions()
        }
        #[cfg(not(target_os = "macos"))]
        {
            Ok(())
        }
    }

    /// Update the permissions checked status
    pub fn set_permissions_checked(&mut self, checked: bool) {
        self.permissions_checked = checked;
    }

    /// Setup system tray menu callbacks
    #[cfg(target_os = "macos")]
    fn setup_system_tray_callbacks(&mut self) -> Result<(), String> {
        use crate::platform::SystemTrayMenuItemKey;
        
        if let Some(ref system_tray) = self.system_tray {
            // Show UI callback
            system_tray.set_menu_item_callback(SystemTrayMenuItemKey::ShowUI, || {
                println!("System tray: Show UI clicked");
                crate::send_system_tray_event(crate::SystemTrayEvent::ShowUI);
            });

            // Enable/Disable Vietnamese input callback
            system_tray.set_menu_item_callback(SystemTrayMenuItemKey::Enable, || {
                println!("System tray: Toggle Vietnamese input");
                crate::send_system_tray_event(crate::SystemTrayEvent::ToggleVietnamese);
            });

            // Switch to Telex input method
            system_tray.set_menu_item_callback(SystemTrayMenuItemKey::TypingMethodTelex, || {
                println!("System tray: Switch to Telex");
                // Rebuild keyboard layout when input type changes
                crate::platform::rebuild_keyboard_layout_map();
                crate::send_system_tray_event(crate::SystemTrayEvent::SetInputTypeTelex);
            });

            // Switch to VNI input method
            system_tray.set_menu_item_callback(SystemTrayMenuItemKey::TypingMethodVNI, || {
                println!("System tray: Switch to VNI");
                // Rebuild keyboard layout when input type changes
                crate::platform::rebuild_keyboard_layout_map();
                crate::send_system_tray_event(crate::SystemTrayEvent::SetInputTypeVNI);
            });

            // Exit application callback
            system_tray.set_menu_item_callback(SystemTrayMenuItemKey::Exit, || {
                println!("System tray: Exit application");
                std::process::exit(0);
            });

            // Update the initial state of menu items
            self.update_system_tray_state();
            self.update_system_tray_title();
        }
        
        Ok(())
    }

    /// Update system tray menu items to reflect current app state
    #[cfg(target_os = "macos")]
    fn update_system_tray_state(&self) {
        use crate::platform::SystemTrayMenuItemKey;
        
        if let Some(ref system_tray) = self.system_tray {
            // Update Vietnamese input toggle state
            let vietnamese_enabled = self.config.is_vietnamese_enabled();
            let enable_text = if vietnamese_enabled {
                "Tắt gõ tiếng việt"
            } else {
                "Bật gõ tiếng việt"
            };
            system_tray.set_menu_item_title(SystemTrayMenuItemKey::Enable, enable_text);

            // Update input method indicators
            match self.config.input_type {
                crate::core::InputType::Telex => {
                    system_tray.set_menu_item_title(SystemTrayMenuItemKey::TypingMethodTelex, "Telex ✓");
                    system_tray.set_menu_item_title(SystemTrayMenuItemKey::TypingMethodVNI, "VNI");
                }
                crate::core::InputType::VNI => {
                    system_tray.set_menu_item_title(SystemTrayMenuItemKey::TypingMethodTelex, "Telex");
                    system_tray.set_menu_item_title(SystemTrayMenuItemKey::TypingMethodVNI, "VNI ✓");
                }
                _ => {
                    system_tray.set_menu_item_title(SystemTrayMenuItemKey::TypingMethodTelex, "Telex");
                    system_tray.set_menu_item_title(SystemTrayMenuItemKey::TypingMethodVNI, "VNI");
                }
            }

            // Note: SystemTray::set_title requires &mut self, so we can't call it here
            // This is handled by the update_system_tray_title method instead
        }
    }

    #[cfg(not(target_os = "macos"))]
    fn setup_system_tray_callbacks(&mut self) -> Result<(), String> {
        Ok(())
    }

    /// Update system tray title based on current state
    #[cfg(target_os = "macos")]
    pub fn update_system_tray_title(&mut self) {
        if let Some(ref mut system_tray) = self.system_tray {
            let vietnamese_enabled = self.config.is_vietnamese_enabled();
            let title = if vietnamese_enabled {
                match self.config.input_type {
                    crate::core::InputType::Telex => "VN",
                    crate::core::InputType::VNI => "VN",
                    _ => "VN",
                }
            } else {
                "EN"
            };
            system_tray.set_title(title);
        }
    }

    #[cfg(not(target_os = "macos"))]
    pub fn update_system_tray_title(&mut self) {
        // No-op for non-macOS platforms
    }

    /// Process pending system tray events
    pub fn process_system_tray_events(&mut self) {
        let mut events = Vec::new();
        
        // Collect all pending events first
        if let Some(ref receiver) = self.system_tray_receiver {
            while let Ok(event) = receiver.try_recv() {
                events.push(event);
            }
        }
        
        // Process the events
        for event in events {
            match event {
                crate::SystemTrayEvent::ShowUI => {
                    println!("Processing system tray event: Show UI");
                    // TODO: Implement showing the main window
                }
                crate::SystemTrayEvent::ToggleVietnamese => {
                    println!("Processing system tray event: Toggle Vietnamese");
                    self.toggle_vietnamese_input();
                }
                crate::SystemTrayEvent::SetInputTypeTelex => {
                    println!("Processing system tray event: Set input type Telex");
                    self.set_input_type(InputType::Telex);
                }
                crate::SystemTrayEvent::SetInputTypeVNI => {
                    println!("Processing system tray event: Set input type VNI");
                    self.set_input_type(InputType::VNI);
                }
            }
        }
    }

    fn render_dropdown(&mut self, label: &str, options: &[&str], selected_index: usize, dropdown_type: &str, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let label = label.to_string();
        
        // Convert options to Vec<String> which implements DropdownItem
        let dropdown_options: Vec<String> = options.iter().map(|&s| s.to_string()).collect();
        
        // Get or create the appropriate dropdown state
        let dropdown_state = match dropdown_type {
            "input_type" => {
                if self.input_type_dropdown.is_none() {
                    let state = cx.new(|cx| DropdownState::new(dropdown_options, Some(selected_index), window, cx));
                    let _ = cx.subscribe_in(&state, window, Self::on_input_type_dropdown_event);
                    self.input_type_dropdown = Some(state.clone());
                    state
                } else {
                    self.input_type_dropdown.as_ref().unwrap().clone()
                }
            }
            "encoding" => {
                if self.encoding_dropdown.is_none() {
                    let state = cx.new(|cx| DropdownState::new(dropdown_options, Some(selected_index), window, cx));
                    let _ = cx.subscribe_in(&state, window, Self::on_encoding_dropdown_event);
                    self.encoding_dropdown = Some(state.clone());
                    state
                } else {
                    self.encoding_dropdown.as_ref().unwrap().clone()
                }
            }
            _ => {
                // Fallback for unknown dropdown types
                cx.new(|cx| DropdownState::new(dropdown_options, Some(selected_index), window, cx))
            }
        };
        
        div()
            .flex()
            .items_center()
            .gap_2()
            .child(
                div()
                    .text_color(rgb(0xe2e8f0))
                    .text_sm()
                    .w_16()
                    .child(label)
            )
            .child(
                // Use gpui-component Dropdown with proper state
                Dropdown::new(&dropdown_state).cleanable()
                    .placeholder("Select...")
            )
    }

    fn on_input_type_dropdown_event(
        &mut self,
        _: &Entity<DropdownState<Vec<String>>>,
        event: &DropdownEvent<Vec<String>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            DropdownEvent::Confirm(value) => {
                println!("Selected input type: {:?}", value);
                // Convert the selected string to InputType and update config
                if let Some(val) = value {
                    let input_type = match val.as_str() {
                        "Telex" => InputType::Telex,
                        "VNI" => InputType::VNI,
                        "VIQR" => InputType::VIQR,
                        _ => InputType::Telex, // Default fallback
                    };
                    self.set_input_type(input_type);
                    cx.notify();
                }
            }
        }
    }

    fn on_encoding_dropdown_event(
        &mut self,
        _: &Entity<DropdownState<Vec<String>>>,
        event: &DropdownEvent<Vec<String>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            DropdownEvent::Confirm(value) => {
                println!("Selected encoding: {:?}", value);
                // Convert the selected string to Encoding and update config
                if let Some(val) = value {
                    let encoding = match val.as_str() {
                        "Unicode" => Encoding::Unicode,
                        "TCVN3" => Encoding::TCVN3,
                        "VNI-Win" => Encoding::VNIWin,
                        _ => Encoding::Unicode, // Default fallback
                    };
                    self.set_encoding(encoding);
                    cx.notify();
                }
            }
        }
    }

    fn render_checkbox(&self, label: &str, checked: bool) -> impl IntoElement {
        let label = label.to_string();
        div()
            .flex()
            .items_center()
            .gap_3()
            .cursor_pointer()
            .child(
                div()
                    .size_4()
                    .rounded_sm()
                    .border_1()
                    .border_color(rgb(0x718096))
                    .flex()
                    .items_center()
                    .justify_center()
                    .when(checked, |this| {
                        this.bg(rgb(0x3182ce))
                            .border_color(rgb(0x3182ce))
                            .child(
                                div()
                                    .text_color(rgb(0xffffff))
                                    .text_xs()
                                    .child("✓")
                            )
                    })
                    .when(!checked, |this| {
                        this.bg(rgb(0x2d3748))
                    })
            )
            .child(
                div()
                    .text_color(rgb(0xe2e8f0))
                    .text_sm()
                    .child(label)
            )
    }

    fn render_vietnamese_toggle(&self) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap_4()
            .child(
                div()
                    .cursor_pointer()
                    .on_mouse_down(MouseButton::Left, {
                        move |_, _, _| {
                            println!("Vietnamese mode clicked");
                            // For now, just log - proper state update would need context
                        }
                    })
                    .child(self.render_radio_button(
                        "Tiếng Việt",
                        matches!(self.config.input_mode, InputMode::Vietnamese)
                    ))
            )
            .child(
                div()
                    .cursor_pointer()
                    .on_mouse_down(MouseButton::Left, {
                        move |_, _, _| {
                            println!("English mode clicked");
                            // For now, just log - proper state update would need context
                        }
                    })
                    .child(self.render_radio_button(
                        "English",
                        matches!(self.config.input_mode, InputMode::English)
                    ))
            )
    }

    fn render_radio_button(&self, label: &str, selected: bool) -> impl IntoElement {
        let label = label.to_string();
        div()
            .flex()
            .items_center()
            .gap_3()
            .child(
                div()
                    .size_4()
                    .rounded_full()
                    .border_1()
                    .border_color(rgb(0x718096))
                    .flex()
                    .items_center()
                    .justify_center()
                    .when(selected, |this| {
                        this.border_color(rgb(0x3182ce))
                            .child(
                                div()
                                    .size_2()
                                    .rounded_full()
                                    .bg(rgb(0x3182ce))
                            )
                    })
                    .when(!selected, |this| {
                        this.bg(rgb(0x2d3748))
                    })
            )
            .child(
                div()
                    .text_color(rgb(0xe2e8f0))
                    .text_sm()
                    .child(label)
            )
    }

    fn render_button(&self, label: &str, is_primary: bool) -> impl IntoElement {
        let label = label.to_string();
        div()
            .px_4()
            .py_2()
            .rounded_md()
            .cursor_pointer()
            .when(is_primary, |this| {
                this.bg(rgb(0x3182ce))
                    .text_color(rgb(0xffffff))
                    .hover(|this| this.bg(rgb(0x2c5aa0)))
            })
            .when(!is_primary, |this| {
                this.bg(rgb(0x4a5568))
                    .text_color(rgb(0xe2e8f0))
                    .hover(|this| this.bg(rgb(0x5a6c7d)))
            })
            .child(label)
    }

    fn render_clickable_button(&self, label: &str, is_primary: bool, action: &'static str) -> impl IntoElement {
        let label = label.to_string();
        div()
            .px_6()
            .py_2()
            .min_w_24()
            .text_center()
            .rounded_md()
            .cursor_pointer()
            .text_sm()
            .on_mouse_down(MouseButton::Left, {
                let action = action;
                move |_, _, cx| {
                    match action {
                        "exit" => {
                            println!("Exit button clicked - closing application");
                            cx.quit();
                        }
                        "ok" => {
                            println!("OK button clicked - saving configuration and closing");
                            cx.quit();
                        }
                        "default" => {
                            println!("Default button clicked - resetting to default configuration");
                            // For now, just log - proper state update would need context
                        }
                        _ => {}
                    }
                }
            })
            .when(is_primary, |this| {
                this.bg(rgb(0x3182ce))
                    .text_color(rgb(0xffffff))
                    .hover(|this| this.bg(rgb(0x2c5aa0)))
            })
            .when(!is_primary, |this| {
                this.bg(rgb(0x4a5568))
                    .text_color(rgb(0xe2e8f0))
                    .hover(|this| this.bg(rgb(0x5a6c7d)))
            })
            .child(label)
    }

    fn render_hotkey_config(&self) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap_3()
            .mb_3()
            .child(
                div()
                    .text_color(rgb(0xe2e8f0))
                    .text_sm()
                    .min_w_20()
                    .child("Phím tắt:")
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_3()
                    .py_2()
                    .bg(rgb(0x2d3748))
                    .border_1()
                    .border_color(rgb(0x718096))
                    .rounded_md()
                    .cursor_pointer()
                    .hover(|this| this.bg(rgb(0x374151)))
                    .min_w_40()
                    .on_mouse_down(MouseButton::Left, {
                        move |_, _, _| {
                            println!("Hotkey config clicked - cycling hotkeys");
                            // For now, just log - proper state update would need context
                        }
                    })
                    .child(
                        div()
                            .text_color(rgb(0xe2e8f0))
                            .text_sm()
                            .child(self.config.get_hotkey_description())
                    )
                    .child(
                        div()
                            .text_color(rgb(0xa0aec0))
                            .text_xs()
                            .ml_2()
                            .child("▼")
                    )
            )
    }

    fn render_control_section(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .bg(rgb(0x4a5568))
            .rounded_lg()
            .p_3()
            .mb_3()
            .child(
                div()
                    .text_color(rgb(0xe2e8f0))
                    .text_base()
                    .mb_2()
                    .child("Điều khiển")
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_8()
                    .mb_3()
                    .child({
                        let input_type_index = match self.config.input_type {
                            InputType::Telex => 0,
                            InputType::VNI => 1,
                            InputType::VIQR => 2,
                        };
                        self.render_dropdown(
                            "Kiểu gõ:",
                            &["Telex", "VNI", "VIQR"],
                            input_type_index,
                            "input_type",
                            window,
                            cx
                        )
                    })
                    .child({
                        let encoding_index = match self.config.encoding {
                            Encoding::Unicode => 0,
                            Encoding::TCVN3 => 1,
                            Encoding::VNIWin => 2,
                        };
                        self.render_dropdown(
                            "Bảng mã:",
                            &["Unicode", "TCVN3", "VNI-Win"],
                            encoding_index,
                            "encoding",
                            window,
                            cx
                        )
                    })
            )
            .child(self.render_hotkey_config())
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .mb_3()
                    .child(
                        div()
                            .text_color(rgb(0xe2e8f0))
                            .text_sm()
                            .min_w_20()
                            .child("Phím chuyển:")
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(self.render_checkbox("^", self.config.keyboard.shift_enabled))
                            .child(self.render_checkbox("⌃", self.config.keyboard.ctrl_enabled))
                            .child(self.render_checkbox("⌘", self.config.keyboard.cmd_enabled))
                            .child(self.render_checkbox("⌂", self.config.keyboard.home_enabled))
                            .child(
                                div()
                                    .bg(rgb(0x3182ce))
                                    .px_1()
                                    .py_1()
                                    .rounded_sm()
                                    .text_color(rgb(0xffffff))
                                    .text_xs()
                                    .child("I")
                            )
                            .child(self.render_checkbox("Kêu beep", self.config.keyboard.beep_enabled))
                    )
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .text_color(rgb(0xe2e8f0))
                            .text_sm()
                            .min_w_20()
                            .child("Chế độ gõ:")
                    )
                    .child(self.render_vietnamese_toggle())
            )
    }

    fn render_tabs(&self) -> impl IntoElement {
        div()
            .flex()
            .gap_1()
            .mb_3()
            .child(self.render_button("Bộ gõ", true))
            .child(self.render_button("Gõ tắt", false))
            .child(self.render_button("Hệ thống", false))
            .child(self.render_button("Thông tin", false))
    }

    fn render_advanced_settings(&self) -> impl IntoElement {
        div()
            .bg(rgb(0x4a5568))
            .rounded_lg()
            .p_3()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .gap_4()
                            .child(
                                div()
                                    .flex_1()
                                    .child(self.render_checkbox("Đặt dấu òa, úy (thay vì òa, úy)", self.config.advanced.replace_oa_uy))
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .child(self.render_checkbox("Kiểm tra chính tả", self.config.advanced.spell_check))
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .gap_8()
                            .child(
                                div()
                                    .flex_1()
                                    .child(self.render_checkbox("Sửa lỗi gõ ý (trình duyệt, Excel,...)", self.config.advanced.auto_restart_typos))
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .child(self.render_checkbox("Tự khởi phục phím với tự sai", self.config.advanced.auto_restart_typos))
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .gap_8()
                            .child(
                                div()
                                    .flex_1()
                                    .child(self.render_checkbox("Viết Hoa chữ cái đầu câu", self.config.advanced.vietnamese_capital))
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .child(self.render_checkbox("Cho phép \"z w j f\" làm phụ âm", self.config.advanced.allow_silent_consonants))
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .gap_8()
                            .child(
                                div()
                                    .flex_1()
                                    .child(self.render_checkbox("Chuyển chế độ thông minh", self.config.advanced.smart_switching))
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .child(self.render_checkbox("Tạm tắt chính tả bằng phím ^", self.config.advanced.temp_disable_spell_check))
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .gap_8()
                            .child(
                                div()
                                    .flex_1()
                                    .child(self.render_checkbox("Tự ghi nhớ bảng mã theo ứng dụng", self.config.advanced.remember_encoding))
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .child(self.render_checkbox("Tạm tắt VKey bằng phím ⌘", self.config.advanced.temp_disable_openkey))
                            )
                    )
            )
    }

    fn render_bottom_buttons(&self) -> impl IntoElement {
        div()
            .flex()
            .justify_center()
            .items_center()
            .gap_4()
            .mt_6()
            .mb_4()
            .child(self.render_clickable_button("✕ Kết thúc", false, "exit"))
            .child(self.render_clickable_button("👍 Mặc định", false, "default"))
            .child(self.render_clickable_button("✓ OK", true, "ok"))
    }
}

impl Render for VKeyApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Process any pending system tray events
        self.process_system_tray_events();
        div()
            .flex()
            .flex_col()
            .bg(rgb(0x2d3748))
            .w_full()
            .h_full()
            .p_4()
            .child(
                // Title bar
                div()
                    .text_color(rgb(0xe2e8f0))
                    .text_lg()
                    .text_center()
                    .mb_4()
                    .child("VKey - Bộ gõ Tiếng Việt")
            )
            .child(self.render_control_section(window, cx))
            .child(self.render_tabs())
            .child(self.render_advanced_settings())
            .child(self.render_bottom_buttons())
    }
} 