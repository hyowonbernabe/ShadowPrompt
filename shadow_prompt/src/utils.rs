#[derive(Debug, PartialEq, Clone)]
pub enum McqAnswer {
    A,
    B,
    C,
    D,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub enum QuestionType {
    MultipleChoice(McqAnswer),
    TrueFalse(bool),
    Identification(String),
    Unknown,
}

pub fn parse_mcq_answer(text: &str) -> Option<McqAnswer> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }

    // Check for exact matches or "Answer: X" patterns
    // We'll normalize to lowercase checks
    let lower = text.to_lowercase();

    // 1. Check for single character answers: "A", "a", "A.", "a)"
    // Or "1", "2", "3", "4"
    if let Some(ans) = check_single_token(&lower) {
        return Some(ans);
    }

    // 2. Check for starts with "A.", "A)", "1.", "1)"
    if let Some(ans) = check_starts_with(&lower) {
        return Some(ans);
    }

    // 3. Check for "Answer: A" pattern
    if let Some(pos) = lower.find("answer:") {
        let remainder = &lower[pos + 7..].trim();
        if let Some(ans) = check_single_token(remainder) {
            return Some(ans);
        }
        if let Some(ans) = check_starts_with(remainder) {
            return Some(ans);
        }
    }

    None
}

fn check_single_token(text: &str) -> Option<McqAnswer> {
    // Remove trailing punctuation like '.' or ')'
    let valid_endings = ['.', ')'];
    let clean = text.trim_end_matches(&valid_endings[..]);

    match clean {
        "a" | "1" => Some(McqAnswer::A),
        "b" | "2" => Some(McqAnswer::B),
        "c" | "3" => Some(McqAnswer::C),
        "d" | "4" => Some(McqAnswer::D),
        _ => None,
    }
}

fn check_starts_with(text: &str) -> Option<McqAnswer> {
    if text.starts_with("a.")
        || text.starts_with("a)")
        || text.starts_with("1.")
        || text.starts_with("1)")
    {
        return Some(McqAnswer::A);
    }
    if text.starts_with("b.")
        || text.starts_with("b)")
        || text.starts_with("2.")
        || text.starts_with("2)")
    {
        return Some(McqAnswer::B);
    }
    if text.starts_with("c.")
        || text.starts_with("c)")
        || text.starts_with("3.")
        || text.starts_with("3)")
    {
        return Some(McqAnswer::C);
    }
    if text.starts_with("d.")
        || text.starts_with("d)")
        || text.starts_with("4.")
        || text.starts_with("4)")
    {
        return Some(McqAnswer::D);
    }
    None
}
pub fn parse_mcq_with_context(input: &str, output: &str) -> Option<McqAnswer> {
    let choices = extract_mcq_choices(input);

    if !choices.is_empty() {
        let output_clean = output.trim().to_lowercase();

        for (letter, value) in &choices {
            let value_lower = value.to_lowercase();
            if output_clean == value_lower
                || output_clean.starts_with(&value_lower)
                || value_lower.starts_with(&output_clean)
            {
                return Some(letter.clone());
            }
        }
    }

    parse_mcq_answer(output)
}

#[allow(dead_code)]
pub fn parse_question_type(input: &str, output: &str) -> QuestionType {
    let output_lower = output.to_lowercase();

    // Check for TYPE: prefix from LLM
    if let Some(start) = output_lower.find("type:") {
        let remainder = &output_lower[start..];
        if remainder.starts_with("type:mcq") {
            if let Some(ans) = parse_mcq_answer(output) {
                return QuestionType::MultipleChoice(ans);
            }
        } else if remainder.starts_with("type:tf") {
            if remainder.contains("answer:true") || remainder.contains("answer: t") {
                return QuestionType::TrueFalse(true);
            } else if remainder.contains("answer:false") || remainder.contains("answer: f") {
                return QuestionType::TrueFalse(false);
            }
        } else if remainder.starts_with("type:id") {
            if let Some(ans_start) = remainder.find("answer:") {
                let answer_text = remainder[ans_start + 7..].trim().to_string();
                if !answer_text.is_empty() {
                    return QuestionType::Identification(answer_text);
                }
            }
        }
    }

    // Fallback: Try existing MCQ parsing with context
    if let Some(ans) = parse_mcq_with_context(input, output) {
        return QuestionType::MultipleChoice(ans);
    }

    // Fallback: Try simple True/False detection
    let output_trimmed = output.trim().to_lowercase();
    if output_trimmed == "true"
        || output_trimmed == "t"
        || output_trimmed == "false"
        || output_trimmed == "f"
    {
        return QuestionType::TrueFalse(output_trimmed == "true" || output_trimmed == "t");
    }

    // Fallback: If there's a substantial answer text, treat as Identification
    let answer_only = output.trim();
    if !answer_only.is_empty() && answer_only.len() < 100 {
        return QuestionType::Identification(answer_only.to_string());
    }

    QuestionType::Unknown
}

#[allow(dead_code)]
pub fn question_type_to_display_text(qt: &QuestionType, input: &str) -> String {
    match qt {
        QuestionType::MultipleChoice(ans) => {
            // Try to get the answer text from choices
            let choices = extract_mcq_choices(input);
            for (letter, value) in &choices {
                if *letter == ans.clone() {
                    return format!("{:?}: {}", ans, value);
                }
            }
            format!("{:?}", ans)
        }
        QuestionType::TrueFalse(true) => "True".to_string(),
        QuestionType::TrueFalse(false) => "False".to_string(),
        QuestionType::Identification(text) => text.clone(),
        QuestionType::Unknown => String::new(),
    }
}

fn extract_mcq_choices(input: &str) -> Vec<(McqAnswer, String)> {
    use regex::Regex;

    let mut choices = Vec::new();

    let patterns = [
        r"(?i)\bA[.\)]\s*([^\n\r]+?)(?:\s+B[.\)]|\n|\r|$)",
        r"(?i)\bB[.\)]\s*([^\n\r]+?)(?:\s+C[.\)]|\n|\r|$)",
        r"(?i)\bC[.\)]\s*([^\n\r]+?)(?:\s+D[.\)]|\n|\r|$)",
        r"(?i)\bD[.\)]\s*([^\n\r]+?)(?:\s+E[.\)]|\n|\r|$)",
        r"(?i)\b1[.\)]\s*([^\n\r]+?)(?:\s+2[.\)]|\n|\r|$)",
        r"(?i)\b2[.\)]\s*([^\n\r]+?)(?:\s+3[.\)]|\n|\r|$)",
        r"(?i)\b3[.\)]\s*([^\n\r]+?)(?:\s+4[.\)]|\n|\r|$)",
        r"(?i)\b4[.\)]\s*([^\n\r]+?)(?:\s+5[.\)]|\n|\r|$)",
    ];

    let letter_map = [
        McqAnswer::A,
        McqAnswer::B,
        McqAnswer::C,
        McqAnswer::D,
        McqAnswer::A,
        McqAnswer::B,
        McqAnswer::C,
        McqAnswer::D,
    ];

    for (i, pattern) in patterns.iter().enumerate() {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(input) {
                if let Some(m) = caps.get(1) {
                    let value = m.as_str().trim().to_string();
                    if !value.is_empty() {
                        choices.push((letter_map[i].clone(), value));
                    }
                }
            }
        }
    }

    choices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcq_parsing() {
        assert_eq!(parse_mcq_answer("A"), Some(McqAnswer::A));
        assert_eq!(parse_mcq_answer("b"), Some(McqAnswer::B));
        assert_eq!(parse_mcq_answer("3"), Some(McqAnswer::C));
        assert_eq!(parse_mcq_answer("4."), Some(McqAnswer::D));
        assert_eq!(
            parse_mcq_answer("A. This is the answer"),
            Some(McqAnswer::A)
        );
        assert_eq!(parse_mcq_answer("Answer: B"), Some(McqAnswer::B));
        assert_eq!(parse_mcq_answer("The answer is C"), None);
        assert_eq!(parse_mcq_answer("1) Paris"), Some(McqAnswer::A));
    }

    #[test]
    fn test_mcq_with_context() {
        assert_eq!(
            parse_mcq_with_context("What is 2+2? A. 4 B. 3 C. 2 D. 1", "4"),
            Some(McqAnswer::A)
        );
        assert_eq!(
            parse_mcq_with_context("Capital? A) Paris B) London C) Berlin D) Rome", "Paris"),
            Some(McqAnswer::A)
        );
        assert_eq!(
            parse_mcq_with_context("Q1: 1) Red 2) Blue 3) Green 4) Yellow", "Blue"),
            Some(McqAnswer::B)
        );
        assert_eq!(
            parse_mcq_with_context("Normal question without choices", "some answer"),
            None
        );
        assert_eq!(
            parse_mcq_with_context("A. 4 B. 3 C. 2 D. 1", "A) 4"),
            Some(McqAnswer::A)
        );
    }

    #[test]
    fn test_question_type_mcq() {
        let result = parse_question_type("What is 2+2? A. 4 B. 3", "TYPE:MCQ ANSWER:A");
        assert!(matches!(result, QuestionType::MultipleChoice(McqAnswer::A)));
    }

    #[test]
    fn test_question_type_true_false() {
        let result = parse_question_type("True or False: The sky is blue.", "TYPE:TF ANSWER:TRUE");
        assert!(matches!(result, QuestionType::TrueFalse(true)));
    }

    #[test]
    fn test_question_type_identification() {
        let result = parse_question_type("What is the capital of France?", "TYPE:ID ANSWER:Paris");
        assert!(matches!(result, QuestionType::Identification(x) if x == "paris"));
    }

    #[test]
    fn test_hex_parsing() {
        assert_eq!(parse_hex_color("#FF0000"), 0x000000FF);
        assert_eq!(parse_hex_color("#00FF00"), 0x0000FF00);
        assert_eq!(parse_hex_color("#00FFFF"), 0x00FFFF00);
        assert_eq!(parse_hex_color("ZZZ"), 0x00000000);
    }

    #[test]
    fn test_key_parsing() {
        use rdev::Key;
        let keys = crate::utils::parse_keys("A");
        assert_eq!(keys, vec![Key::KeyA]);

        let keys = crate::utils::parse_keys("Ctrl+Z");
        assert_eq!(keys, vec![Key::ControlLeft, Key::KeyZ]);

        let keys = crate::utils::parse_keys("Ctrl+Shift+Esc");
        assert_eq!(keys, vec![Key::ControlLeft, Key::ShiftLeft, Key::Escape]);

        let keys = crate::utils::parse_keys("F12");
        assert_eq!(keys, vec![Key::F12]);

        let keys = crate::utils::parse_keys("  Alt +  Tab ");
        assert_eq!(keys, vec![Key::Alt, Key::Tab]);
    }
}

pub fn parse_hex_color(hex: &str) -> u32 {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return 0x00000000; // Default Black or Transparent if invalid
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

    // Windows COLORREF is 0x00BBGGRR
    ((b as u32) << 16) | ((g as u32) << 8) | (r as u32)
}

use rdev::Key;

pub fn parse_keys(config_str: &str) -> Vec<Key> {
    let mut keys = Vec::new();
    for part in config_str.split('+') {
        let trimmed = part.trim().to_lowercase();
        match trimmed.as_str() {
            // Modifiers
            "ctrl" | "control" => keys.push(Key::ControlLeft), // Simplifying to Left for now
            "shift" => keys.push(Key::ShiftLeft),
            "alt" => keys.push(Key::Alt),
            "meta" | "win" | "super" => keys.push(Key::MetaLeft),

            // Functional
            "space" => keys.push(Key::Space),
            "enter" | "return" => keys.push(Key::Return),
            "esc" | "escape" => keys.push(Key::Escape),
            "tab" => keys.push(Key::Tab),
            "backspace" => keys.push(Key::Backspace),
            "capslock" => keys.push(Key::CapsLock),

            // F-Keys
            "f1" => keys.push(Key::F1),
            "f2" => keys.push(Key::F2),
            "f3" => keys.push(Key::F3),
            "f4" => keys.push(Key::F4),
            "f5" => keys.push(Key::F5),
            "f6" => keys.push(Key::F6),
            "f7" => keys.push(Key::F7),
            "f8" => keys.push(Key::F8),
            "f9" => keys.push(Key::F9),
            "f10" => keys.push(Key::F10),
            "f11" => keys.push(Key::F11),
            "f12" => keys.push(Key::F12),

            // Letters
            "a" => keys.push(Key::KeyA),
            "b" => keys.push(Key::KeyB),
            "c" => keys.push(Key::KeyC),
            "d" => keys.push(Key::KeyD),
            "e" => keys.push(Key::KeyE),
            "f" => keys.push(Key::KeyF),
            "g" => keys.push(Key::KeyG),
            "h" => keys.push(Key::KeyH),
            "i" => keys.push(Key::KeyI),
            "j" => keys.push(Key::KeyJ),
            "k" => keys.push(Key::KeyK),
            "l" => keys.push(Key::KeyL),
            "m" => keys.push(Key::KeyM),
            "n" => keys.push(Key::KeyN),
            "o" => keys.push(Key::KeyO),
            "p" => keys.push(Key::KeyP),
            "q" => keys.push(Key::KeyQ),
            "r" => keys.push(Key::KeyR),
            "s" => keys.push(Key::KeyS),
            "t" => keys.push(Key::KeyT),
            "u" => keys.push(Key::KeyU),
            "v" => keys.push(Key::KeyV),
            "w" => keys.push(Key::KeyW),
            "x" => keys.push(Key::KeyX),
            "y" => keys.push(Key::KeyY),
            "z" => keys.push(Key::KeyZ),

            // Numbers
            "0" => keys.push(Key::Num0),
            "1" => keys.push(Key::Num1),
            "2" => keys.push(Key::Num2),
            "3" => keys.push(Key::Num3),
            "4" => keys.push(Key::Num4),
            "5" => keys.push(Key::Num5),
            "6" => keys.push(Key::Num6),
            "7" => keys.push(Key::Num7),
            "8" => keys.push(Key::Num8),
            "9" => keys.push(Key::Num9),

            _ => {
                eprintln!("Warning: Unknown key in config: {}", part);
            }
        }
    }
    keys
}
