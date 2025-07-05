use vi::{VNI, TELEX, TransformResult};
use crate::core::types::InputType;

#[derive(Debug, Clone)]
pub struct VietnameseInputProcessor {
    /// Raw input buffer (what the user actually typed)
    typing_buffer: String,
    /// Display buffer (what's currently shown on screen after transformation)
    display_buffer: String,
    input_type: InputType,
    /// Track if we should continue processing characters
    should_track: bool,
    /// Previous word for restoration purposes
    previous_word: String,
    /// Maximum word length to prevent infinite growth
    max_word_length: usize,
}

impl VietnameseInputProcessor {
    pub fn new(input_type: InputType) -> Self {
        Self {
            typing_buffer: String::new(),
            display_buffer: String::new(),
            input_type,
            should_track: true,
            previous_word: String::new(),
            max_word_length: 10, // Maximum possible word length
        }
    }

    pub fn set_input_type(&mut self, input_type: InputType) {
        self.input_type = input_type;
        // Clear buffers when switching input types
        self.clear_buffer();
    }

    pub fn process_key(&mut self, key: char) -> ProcessingResult {
        // Handle special keys
        match key {
            '\u{8}' => return self.handle_backspace(), // Backspace
            '\r' | '\n' => return self.handle_enter(),   // Enter
            ' ' => return self.handle_space(),           // Space
            '\t' => return self.handle_tab(),            // Tab
            '\u{1B}' => return self.handle_escape(),     // Escape
            _ => {}
        }

        // Only process printable ASCII characters for Vietnamese input
        if !key.is_ascii() || key.is_ascii_control() {
            return ProcessingResult::PassThrough(key);
        }

        // Check if we should stop tracking
        if !self.should_track {
            return ProcessingResult::PassThrough(key);
        }

        // Handle special characters that should stop tracking
        if "()[]{}<>/\\!@#$%^&*-_=+|~`,.;'\"?".contains(key) {
            self.new_word();
            return ProcessingResult::PassThrough(key);
        }

        // Remove numeric prefix if present
        if let Some(first_char) = self.typing_buffer.chars().next() {
            if first_char.is_numeric() {
                self.typing_buffer.remove(0);
                if !self.display_buffer.is_empty() {
                    self.display_buffer.remove(0);
                }
            }
        }

        // Check max word length
        if self.typing_buffer.len() >= self.max_word_length {
            self.new_word();
            return ProcessingResult::PassThrough(key);
        }

        // Store the current display buffer length for backspace counting
        let previous_display_length = self.display_buffer.chars().count();
        
        // Add character to typing buffer
        self.typing_buffer.push(key);

        // Transform the buffer using vi-rs
        let mut result = String::new();
        let transform_result = match self.input_type {
            InputType::Telex => {
                vi::transform_buffer(&TELEX, self.typing_buffer.chars(), &mut result)
            }
            InputType::VNI => {
                vi::transform_buffer(&VNI, self.typing_buffer.chars(), &mut result)
            }
            InputType::VIQR => {
                // VIQR is not supported by vi-rs, fallback to raw input
                result = self.typing_buffer.clone();
                TransformResult::default()
            }
        };

        // Update display buffer
        self.display_buffer = result.clone();

                    // Check if transformation removed letters or tone marks
        if transform_result.letter_modification_removed || transform_result.tone_mark_removed {
            self.stop_tracking();
        }

        // Check for tone duplicate patterns that should stop tracking
        if self.should_stop_tracking_due_to_patterns() {
            self.stop_tracking();
        }

        ProcessingResult::ProcessedText {
            text: result,
            buffer_length: previous_display_length,
        }
    }

    pub fn handle_backspace(&mut self) -> ProcessingResult {
        if self.typing_buffer.is_empty() {
            return ProcessingResult::PassThrough('\u{8}');
        }

        // Store the current displayed length before modifying buffer
        let previous_display_length = self.display_buffer.chars().count();
        
        // Remove last character from typing buffer
        self.typing_buffer.pop();

        if self.typing_buffer.is_empty() {
            self.clear_buffer();
            return ProcessingResult::ClearAndPassBackspace;
        }

        // Re-transform the remaining buffer
        let mut result = String::new();
        match self.input_type {
            InputType::Telex => {
                vi::transform_buffer(&TELEX, self.typing_buffer.chars(), &mut result);
            }
            InputType::VNI => {
                vi::transform_buffer(&VNI, self.typing_buffer.chars(), &mut result);
            }
            InputType::VIQR => {
                result = self.typing_buffer.clone();
            }
        }

        // Update display buffer
        self.display_buffer = result.clone();
        
        ProcessingResult::ProcessedText {
            text: result,
            buffer_length: previous_display_length,
        }
    }

    fn handle_enter(&mut self) -> ProcessingResult {
        self.new_word();
        ProcessingResult::PassThrough('\n')
    }

    fn handle_tab(&mut self) -> ProcessingResult {
        self.new_word();
        ProcessingResult::PassThrough('\t')
    }

    fn handle_escape(&mut self) -> ProcessingResult {
        // Escape should restore the original typed text
        if !self.typing_buffer.is_empty() {
            let original_text = self.typing_buffer.clone();
            let display_length = self.display_buffer.chars().count();
            self.new_word();
            return ProcessingResult::RestoreText {
                text: original_text,
                buffer_length: display_length,
            };
        }
        ProcessingResult::PassThrough('\u{1B}')
    }

    pub fn handle_space(&mut self) -> ProcessingResult {
        if self.typing_buffer.is_empty() {
            return ProcessingResult::PassThrough(' ');
        }

        // Get the final transformed text
        let mut result = String::new();
        match self.input_type {
            InputType::Telex => {
                vi::transform_buffer(&TELEX, self.typing_buffer.chars(), &mut result);
            }
            InputType::VNI => {
                vi::transform_buffer(&VNI, self.typing_buffer.chars(), &mut result);
            }
            InputType::VIQR => {
                result = self.typing_buffer.clone();
            }
        }
        
        let display_length = self.display_buffer.chars().count();
        
        // Commit the buffer and add space
        self.new_word();
        
        ProcessingResult::ProcessedText {
            text: format!("{} ", result),
            buffer_length: display_length,
        }
    }

    /// Start tracking a new word
    pub fn new_word(&mut self) {
        if !self.typing_buffer.is_empty() {
            self.previous_word = self.typing_buffer.clone();
        }
        self.clear_buffer();
        self.should_track = true;
    }

    /// Stop tracking the current word
    pub fn stop_tracking(&mut self) {
        self.should_track = false;
    }

    /// Check if we should stop tracking due to tone duplicate patterns
    fn should_stop_tracking_due_to_patterns(&self) -> bool {
        // Detect attempts to restore a word by doubling tone marks like ss, rr, ff, jj, xx
        const TONE_DUPLICATE_PATTERNS: [&str; 17] = [
            "ss", "ff", "jj", "rr", "xx", "ww", "kk", "tt", "nn", "mm", "yy", "hh", "ii", "aaa", "eee",
            "ooo", "ddd",
        ];
        
        let buffer_lower = self.typing_buffer.to_ascii_lowercase();
        TONE_DUPLICATE_PATTERNS
            .iter()
            .any(|pattern| buffer_lower.contains(pattern))
    }

    /// Get the backspace count needed to clear the current displayed text
    /// This includes special handling for text selection
    pub fn get_backspace_count(&self, is_delete: bool, has_text_selection: bool) -> usize {
        let display_length = self.display_buffer.chars().count();
        let backspace_count = if is_delete && display_length >= 1 {
            display_length
        } else if display_length > 0 {
            display_length - 1
        } else {
            0
        };

        // Add an extra backspace to compensate for text selection deletion
        // This is useful in applications like Chrome where the URL bar uses text selection
        if has_text_selection {
            backspace_count + 1
        } else {
            backspace_count
        }
    }

    /// Check if the current word should be restored based on validation
    pub fn should_restore_word(&self) -> bool {
        if self.typing_buffer.is_empty() || self.display_buffer.is_empty() {
            return false;
        }

        // If the typing buffer and display buffer are the same, no transformation occurred
        if self.typing_buffer == self.display_buffer {
            return false;
        }

        // Check if the transformed word is valid Vietnamese
        // This would require the vi-rs validation functionality
        // For now, we'll use a simple heuristic
        false
    }

    /// Get the original typed text for restoration
    pub fn get_restore_text(&self) -> String {
        self.typing_buffer.clone()
    }

    pub fn clear_buffer(&mut self) {
        self.typing_buffer.clear();
        self.display_buffer.clear();
    }

    pub fn get_current_buffer(&self) -> &str {
        &self.typing_buffer
    }

    pub fn get_display_buffer(&self) -> &str {
        &self.display_buffer
    }

    pub fn get_previous_word(&self) -> &str {
        &self.previous_word
    }

    pub fn is_tracking(&self) -> bool {
        self.should_track
    }

    pub fn is_buffer_empty(&self) -> bool {
        self.typing_buffer.is_empty()
    }

    pub fn reset(&mut self) {
        self.typing_buffer.clear();
        self.display_buffer.clear();
        self.previous_word.clear();
        self.should_track = true;
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
    /// Restore original text (used for Escape key)
    RestoreText {
        text: String,
        buffer_length: usize,
    },
}  