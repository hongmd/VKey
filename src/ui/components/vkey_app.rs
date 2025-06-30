use gpui::{
    div, prelude::*, rgb, Context, IntoElement, Render, Styled, Window, MouseButton,
};
use crate::core::{AppConfig, InputType, Encoding, InputMode, VietnameseInputProcessor};

#[cfg(target_os = "macos")]
use crate::platform::{MacOSKeyboardHandler, system_integration};



pub struct VKeyApp {
    config: AppConfig,
    vietnamese_processor: VietnameseInputProcessor,
    input_text: String,
    #[cfg(target_os = "macos")]
    keyboard_handler: Option<MacOSKeyboardHandler>,
}

impl VKeyApp {
    pub fn new() -> Self {
        let config = AppConfig::default();
        let vietnamese_processor = VietnameseInputProcessor::new(config.input_type);
        
        #[cfg(target_os = "macos")]
        let keyboard_handler = Some(MacOSKeyboardHandler::new(config.input_type));
        
        Self {
            config,
            vietnamese_processor,
            input_text: String::new(),
            #[cfg(target_os = "macos")]
            keyboard_handler,
        }
    }

    /// Initialize the keyboard system integration
    pub fn initialize_keyboard_system(&mut self) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            // Check accessibility permissions
            if !system_integration::has_accessibility_permissions() {
                system_integration::request_accessibility_permissions()?;
            }
            
            // Install keyboard hook
            system_integration::install_keyboard_hook()?;
            
            println!("Vietnamese input system initialized for macOS");
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
            ProcessingResult::CommitAndPassThrough(ch) => {
                self.vietnamese_processor.clear_buffer();
                self.input_text.push(ch);
                ch.to_string()
            }
        }
    }

    /// Handle input type change
    pub fn set_input_type(&mut self, input_type: InputType) {
        self.config.input_type = input_type;
        self.vietnamese_processor.set_input_type(input_type);
        
        #[cfg(target_os = "macos")]
        if let Some(ref mut handler) = self.keyboard_handler {
            handler.set_input_type(input_type);
        }
    }

    /// Toggle Vietnamese input on/off
    pub fn toggle_vietnamese_input(&mut self) {
        match self.config.input_mode {
            InputMode::Vietnamese => {
                self.config.input_mode = InputMode::English;
                #[cfg(target_os = "macos")]
                if let Some(ref mut handler) = self.keyboard_handler {
                    handler.set_enabled(false);
                }
            }
            InputMode::English => {
                self.config.input_mode = InputMode::Vietnamese;
                #[cfg(target_os = "macos")]
                if let Some(ref mut handler) = self.keyboard_handler {
                    handler.set_enabled(true);
                }
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

    /// Clear the input buffer
    pub fn clear_input_buffer(&mut self) {
        self.vietnamese_processor.clear_buffer();
        self.input_text.clear();
        
        #[cfg(target_os = "macos")]
        if let Some(ref handler) = self.keyboard_handler {
            handler.clear_buffer();
        }
    }



    fn render_dropdown(&mut self, label: &str, options: &[&str], selected_index: usize, dropdown_type: &str) -> impl IntoElement {
        let label = label.to_string();
        let selected_option = options[selected_index].to_string();
        let dropdown_type = dropdown_type.to_string();
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
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_2()
                    .py_1()
                    .bg(rgb(0x4a5568))
                    .border_1()
                    .border_color(rgb(0x718096))
                    .rounded_sm()
                    .cursor_pointer()
                    .hover(|this| this.bg(rgb(0x5a6c7d)))
                    .w_24()
                    .on_mouse_down(MouseButton::Left, {
                        let dropdown_type = dropdown_type.clone();
                        move |this, _, cx| {
                            // Cycle through the dropdown options
                            let _ = this; // silence unused variable warning
                            match dropdown_type.as_str() {
                                "input_type" => {
                                    println!("Input type dropdown clicked - cycling input types");
                                }
                                "encoding" => {
                                    println!("Encoding dropdown clicked - cycling encodings");
                                }
                                _ => {}
                            }

                        }
                    })
                    .child(
                        div()
                            .text_color(rgb(0xe2e8f0))
                            .text_sm()
                            .child(selected_option)
                    )
                    .child(
                        div()
                            .text_color(rgb(0xa0aec0))
                            .text_xs()
                            .ml_1()
                            .child("▼")
                    )
            )
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

    fn render_radio_button(&self, label: &str, selected: bool) -> impl IntoElement {
        let label = label.to_string();
        div()
            .flex()
            .items_center()
            .gap_3()
            .cursor_pointer()
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
                            // We'll implement state reset differently since we can't access self here
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

    fn render_control_section(&mut self) -> impl IntoElement {
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
                            "input_type"
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
                            "encoding"
                        )
                    })
            )
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
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(
                                self.render_radio_button(
                                    "Tiếng Việt",
                                    matches!(self.config.input_mode, InputMode::Vietnamese)
                                )
                            )
                            .child(
                                self.render_radio_button(
                                    "English",
                                    matches!(self.config.input_mode, InputMode::English)
                                )
                            )
                    )
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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
            .child(self.render_control_section())
            .child(self.render_tabs())
            .child(self.render_advanced_settings())
            .child(self.render_bottom_buttons())
    }
} 