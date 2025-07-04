use vi::{VNI, TELEX};
use crate::core::types::InputType;

#[derive(Debug, Clone)]
pub struct VietnameseInputProcessor {
    current_buffer: String,
    processed_text: String,
    input_type: InputType,
    last_displayed_length: usize,
}

impl VietnameseInputProcessor {
    pub fn new(input_type: InputType) -> Self {
        Self {
            current_buffer: String::new(),
            processed_text: String::new(),
            input_type,
            last_displayed_length: 0,
        }
    }

    pub fn set_input_type(&mut self, input_type: InputType) {
        self.input_type = input_type;
        // Clear buffer when switching input types
        self.clear_buffer();
    }

    pub fn process_key(&mut self, key: char) -> ProcessingResult {
        // Handle special keys
        println!("Processing key: {}", key);
        match key {
            '\u{8}' => return self.handle_backspace(), // Backspace
            '\r' | '\n' => return self.handle_enter(),   // Enter
            ' ' => return self.handle_space(),           // Space
            _ => {}
        }

        // Only process printable ASCII characters for Vietnamese input
        if !key.is_ascii() || key.is_ascii_control() {
            return ProcessingResult::PassThrough(key);
        }

        // Add character to buffer
        self.current_buffer.push(key);

        // Transform the buffer using vi-rs
        let mut result = String::new();
        match self.input_type {
            InputType::Telex => {
                vi::transform_buffer(&TELEX, self.current_buffer.chars(), &mut result);
            }
            InputType::VNI => {
                vi::transform_buffer(&VNI, self.current_buffer.chars(), &mut result);
            }
            InputType::VIQR => {
                // VIQR is not supported by vi-rs, fallback to raw input
                result = self.current_buffer.clone();
            }
        }

        let displayed_length = self.last_displayed_length;
        self.last_displayed_length = result.chars().count();
        
        ProcessingResult::ProcessedText {
            text: result,
            buffer_length: displayed_length,
        }
    }

    pub fn handle_backspace(&mut self) -> ProcessingResult {
        if self.current_buffer.is_empty() {
            return ProcessingResult::PassThrough('\u{8}');
        }

        // Remove last character from buffer
        self.current_buffer.pop();

        if self.current_buffer.is_empty() {
            self.last_displayed_length = 0;
            return ProcessingResult::ClearAndPassBackspace;
        }

        // Re-transform the remaining buffer
        let mut result = String::new();
        match self.input_type {
            InputType::Telex => {
                vi::transform_buffer(&TELEX, self.current_buffer.chars(), &mut result);
            }
            InputType::VNI => {
                vi::transform_buffer(&VNI, self.current_buffer.chars(), &mut result);
            }
            InputType::VIQR => {
                result = self.current_buffer.clone();
            }
        }

        let displayed_length = self.last_displayed_length;
        self.last_displayed_length = result.chars().count();
        
        ProcessingResult::ProcessedText {
            text: result,
            buffer_length: displayed_length,
        }
    }

    fn handle_enter(&mut self) -> ProcessingResult {
        self.commit_buffer();
        ProcessingResult::CommitAndPassThrough('\n')
    }

    pub fn handle_space(&mut self) -> ProcessingResult {
        if self.current_buffer.is_empty() {
            return ProcessingResult::PassThrough(' ');
        }

        // Get the transformed text before committing
        let mut result = String::new();
        match self.input_type {
            InputType::Telex => {
                vi::transform_buffer(&TELEX, self.current_buffer.chars(), &mut result);
            }
            InputType::VNI => {
                vi::transform_buffer(&VNI, self.current_buffer.chars(), &mut result);
            }
            InputType::VIQR => {
                result = self.current_buffer.clone();
            }
        }
        
        let displayed_length = self.last_displayed_length;
        
        // Commit the buffer (clears it internally)
        self.commit_buffer();
        self.last_displayed_length = 0; // Reset after space
        
        // Return the committed text with space appended
        ProcessingResult::ProcessedText {
            text: format!("{} ", result),
            buffer_length: displayed_length,
        }
    }

    fn commit_buffer(&mut self) {
        if !self.current_buffer.is_empty() {
            let mut result = String::new();
            match self.input_type {
                InputType::Telex => {
                    vi::transform_buffer(&TELEX, self.current_buffer.chars(), &mut result);
                }
                InputType::VNI => {
                    vi::transform_buffer(&VNI, self.current_buffer.chars(), &mut result);
                }
                InputType::VIQR => {
                    result = self.current_buffer.clone();
                }
            }
            self.processed_text.push_str(&result);
            self.clear_buffer();
        }
    }

    pub fn clear_buffer(&mut self) {
        self.current_buffer.clear();
        self.last_displayed_length = 0;
    }

    pub fn get_current_buffer(&self) -> &str {
        &self.current_buffer
    }

    pub fn get_processed_text(&self) -> &str {
        &self.processed_text
    }

    pub fn reset(&mut self) {
        self.current_buffer.clear();
        self.processed_text.clear();
        self.last_displayed_length = 0;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProcessingResult {
    /// The character should be passed through without processing
    PassThrough(char),
    /// Text has been processed and should replace the current buffer
    ProcessedText {
        text: String,
        buffer_length: usize,
    },
    /// Clear current text and pass backspace through
    ClearAndPassBackspace,
    /// Commit current buffer and pass the character through
    CommitAndPassThrough(char),
}  