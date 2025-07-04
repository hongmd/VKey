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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telex_input() {
        let mut processor = VietnameseInputProcessor::new(InputType::Telex);
        
        // Test "vieset" -> "việt" (correct Telex pattern: ee->ê, et->ệ + t)
        assert_eq!(processor.process_key('v'), ProcessingResult::ProcessedText { text: "v".to_string(), buffer_length: 1 });
        assert_eq!(processor.process_key('i'), ProcessingResult::ProcessedText { text: "vi".to_string(), buffer_length: 2 });
        assert_eq!(processor.process_key('e'), ProcessingResult::ProcessedText { text: "vie".to_string(), buffer_length: 3 });
        assert_eq!(processor.process_key('s'), ProcessingResult::ProcessedText { text: "vié".to_string(), buffer_length: 4 });
        assert_eq!(processor.process_key('e'), ProcessingResult::ProcessedText { text: "viê".to_string(), buffer_length: 5 });
        assert_eq!(processor.process_key('t'), ProcessingResult::ProcessedText { text: "việt".to_string(), buffer_length: 6 });
    }

    #[test]
    fn test_telex_comprehensive() {
        let mut processor = VietnameseInputProcessor::new(InputType::Telex);
        
        // Test vowel combinations
        processor.reset();
        processor.process_key('a');
        processor.process_key('a');
        assert_eq!(processor.get_current_buffer(), "aa");
        let result = processor.process_key(' ');
        // The processed text should have â
        
        // Test tone marks with Telex
        processor.reset();
        processor.process_key('a');
        processor.process_key('s'); // sắc tone
        let result = processor.process_key(' ');
        // Should produce á
        
        processor.reset();
        processor.process_key('a');
        processor.process_key('f'); // huyền tone
        let result = processor.process_key(' ');
        // Should produce à
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

    #[test]
    fn debug_vi_crate_telex() {
        let mut result = String::new();
        let input = "viets";
        vi::transform_buffer(&TELEX, input.chars(), &mut result);
        println!("Input: '{}' -> Output: '{}'", input, result);
        
        // Test correct Telex transformations
        let test_cases = vec![
            "aa", "ee", "oo", "uu", "ii", "aw", "ow", "uw", "dd",
            "as", "af", "ar", "ax", "aj", // tone marks
            "aas", "ees", "oos", // vowel + tone
            "viets", "choaf", "baarn", "naangg",
        ];
        
        for case in test_cases {
            let mut result = String::new();
            vi::transform_buffer(&TELEX, case.chars(), &mut result);
            println!("TELEX: '{}' -> '{}'", case, result);
        }
    }

    #[test]
    fn debug_viet_telex() {
        let test_cases = vec![
            "viees",    // vi + ê + s
            "viees",    // check if es becomes ế
            "viest",    // vi + e + s + t  
            "viets",    // vi + e + t + s
            "vieest",   // vi + ê + s + t
        ];
        
        for case in test_cases {
            let mut result = String::new();
            vi::transform_buffer(&TELEX, case.chars(), &mut result);
            println!("TELEX: '{}' -> '{}'", case, result);
        }
        
        // Let's try step by step
        let mut result = String::new();
        vi::transform_buffer(&TELEX, "viees".chars(), &mut result);
        println!("Expected for việt: viees -> {}", result);
    }

    #[test]
    fn debug_vieset_step_by_step() {
        println!("=== Debug vieset step by step ===");
        let steps = ["v", "vi", "vie", "vies", "viese", "vieset"];
        for step in steps {
            let mut result = String::new();
            vi::transform_buffer(&TELEX, step.chars(), &mut result);
            println!("Step '{}' -> '{}'", step, result);
        }
    }

    #[test]
    fn find_correct_viet_with_circumflex() {
        println!("=== Finding correct pattern for 'việt' (with ê + tone) ===");
        
        // The correct "việt" should have ê with acute tone (ế)
        // Let's test different approaches
        let patterns = [
            // Approach 1: Try to get ê first, then add tone before t
            "viees", "viest", "vieset", "vieest",
            // Approach 2: Try different ordering 
            "vieest", "viesst", "vieeets",
            // Approach 3: Test if there's a way to tone the ê directly
            "vieesst", "vieesxt", "vieesft", "vieesrt", "vieesht"
        ];
        
        for pattern in patterns {
            let mut result = String::new();
            vi::transform_buffer(&TELEX, pattern.chars(), &mut result);
            println!("Pattern '{}' -> '{}'", pattern, result);
            
            // Check if we got the exact target
            if result == "việt" {
                println!("*** FOUND EXACT MATCH: {} ***", pattern);
            }
            
            // Check if we got something with ê and a tone
            if result.contains('ế') || result.contains('ề') || result.contains('ể') || result.contains('ễ') || result.contains('ệ') {
                println!("*** Found ê with tone: {} -> {} ***", pattern, result);
            }
        }
        
        // Let's also test what the expected result should be
        println!("\nTarget: 'việt' contains:");
        for ch in "việt".chars() {
            println!("  '{}' (U+{:04X})", ch, ch as u32);
        }
    }

    #[test]
    fn discover_working_telex_patterns() {
        println!("=== Discovering working Telex patterns ===");
        
        // Let's find patterns that actually work for common Vietnamese words
        let test_words = [
            ("an", vec!["an"]),
            ("anh", vec!["anh"]),
            ("viet", vec!["viet", "viets", "viest", "viets", "vieetf"]),
            ("nam", vec!["nam", "naams"]),
            ("hoa", vec!["hoa", "hoaf", "hoas"]),
            ("que", vec!["que", "ques", "quez"]),
        ];
        
        for (target, variants) in test_words {
            println!("\n--- Testing variants for '{}' ---", target);
            for variant in variants {
                let mut result = String::new();
                vi::transform_buffer(&TELEX, variant.chars(), &mut result);
                println!("  '{}' -> '{}' {}", variant, result, 
                    if result != variant { "✓ transformed" } else { "✗ no change" });
            }
        }
        
        // Test basic Telex rules step by step
        println!("\n=== Basic Telex Rules ===");
        let basic_rules = [
            "aa", "aw", "ee", "oo", "ow", "uu", "uw", "dd",  // vowels and consonants
            "as", "af", "ar", "ax", "aj",  // tones on 'a'
            "es", "ef", "er", "ex", "ej",  // tones on 'e'
            "is", "if", "ir", "ix", "ij",  // tones on 'i'
            "os", "of", "or", "ox", "oj",  // tones on 'o'
            "us", "uf", "ur", "ux", "uj",  // tones on 'u'
        ];
        
        for rule in basic_rules {
            let mut result = String::new();
            vi::transform_buffer(&TELEX, rule.chars(), &mut result);
            if result != rule {
                println!("  '{}' -> '{}'", rule, result);
            }
        }
    }
} 