use vi::{VNI, TELEX};
use crate::core::types::InputType;

#[derive(Debug, Clone)]
pub struct VietnameseInputProcessor {
    current_buffer: String,
    processed_text: String,
    input_type: InputType,
}

impl VietnameseInputProcessor {
    pub fn new(input_type: InputType) -> Self {
        Self {
            current_buffer: String::new(),
            processed_text: String::new(),
            input_type,
        }
    }

    pub fn set_input_type(&mut self, input_type: InputType) {
        self.input_type = input_type;
        // Clear buffer when switching input types
        self.clear_buffer();
    }

    pub fn process_key(&mut self, key: char) -> ProcessingResult {
        // Handle special keys
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

        ProcessingResult::ProcessedText {
            text: result,
            buffer_length: self.current_buffer.len(),
        }
    }

    fn handle_backspace(&mut self) -> ProcessingResult {
        if self.current_buffer.is_empty() {
            return ProcessingResult::PassThrough('\u{8}');
        }

        // Remove last character from buffer
        self.current_buffer.pop();

        if self.current_buffer.is_empty() {
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

        ProcessingResult::ProcessedText {
            text: result,
            buffer_length: self.current_buffer.len(),
        }
    }

    fn handle_enter(&mut self) -> ProcessingResult {
        self.commit_buffer();
        ProcessingResult::CommitAndPassThrough('\n')
    }

    fn handle_space(&mut self) -> ProcessingResult {
        if self.current_buffer.is_empty() {
            return ProcessingResult::PassThrough(' ');
        }

        self.commit_buffer();
        ProcessingResult::CommitAndPassThrough(' ')
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telex_input() {
        let mut processor = VietnameseInputProcessor::new(InputType::Telex);
        
        // Test "viet" -> "việt"
        assert_eq!(processor.process_key('v'), ProcessingResult::ProcessedText { text: "v".to_string(), buffer_length: 1 });
        assert_eq!(processor.process_key('i'), ProcessingResult::ProcessedText { text: "vi".to_string(), buffer_length: 2 });
        assert_eq!(processor.process_key('e'), ProcessingResult::ProcessedText { text: "vie".to_string(), buffer_length: 3 });
        assert_eq!(processor.process_key('t'), ProcessingResult::ProcessedText { text: "viet".to_string(), buffer_length: 4 });
        assert_eq!(processor.process_key('6'), ProcessingResult::ProcessedText { text: "việt".to_string(), buffer_length: 5 });
    }

    #[test]
    fn test_vni_input() {
        let mut processor = VietnameseInputProcessor::new(InputType::VNI);
        
        // Test "viet65" -> "việt"
        assert_eq!(processor.process_key('v'), ProcessingResult::ProcessedText { text: "v".to_string(), buffer_length: 1 });
        assert_eq!(processor.process_key('i'), ProcessingResult::ProcessedText { text: "vi".to_string(), buffer_length: 2 });
        assert_eq!(processor.process_key('e'), ProcessingResult::ProcessedText { text: "vie".to_string(), buffer_length: 3 });
        assert_eq!(processor.process_key('t'), ProcessingResult::ProcessedText { text: "viet".to_string(), buffer_length: 4 });
        assert_eq!(processor.process_key('6'), ProcessingResult::ProcessedText { text: "viết".to_string(), buffer_length: 5 });
        assert_eq!(processor.process_key('5'), ProcessingResult::ProcessedText { text: "việt".to_string(), buffer_length: 6 });
    }

    #[test]
    fn test_backspace() {
        let mut processor = VietnameseInputProcessor::new(InputType::Telex);
        
        processor.process_key('v');
        processor.process_key('i');
        processor.process_key('e');
        processor.process_key('t');
        processor.process_key('6');
        
        // Should have "việt"
        assert_eq!(processor.get_current_buffer(), "viet6");
        
        // Backspace should remove the tone mark
        let result = processor.handle_backspace();
        assert_eq!(result, ProcessingResult::ProcessedText { text: "viet".to_string(), buffer_length: 4 });
    }

    #[test]
    fn test_space_commit() {
        let mut processor = VietnameseInputProcessor::new(InputType::Telex);
        
        processor.process_key('v');
        processor.process_key('i');
        processor.process_key('e');
        processor.process_key('t');
        processor.process_key('6');
        
        let result = processor.handle_space();
        assert_eq!(result, ProcessingResult::CommitAndPassThrough(' '));
        assert_eq!(processor.get_current_buffer(), ""); // Buffer should be cleared
    }
} 