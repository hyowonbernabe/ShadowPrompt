
#[derive(Debug, PartialEq)]
pub enum McqAnswer {
    A,
    B,
    C,
    D,
}

pub fn parse_mcq_answer(text: &str) -> Option<McqAnswer> {
    let text = text.trim();
    if text.is_empty() { return None; }

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
        let remainder = &lower[pos+7..].trim();
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
        _ => None
    }
}

fn check_starts_with(text: &str) -> Option<McqAnswer> {
    if text.starts_with("a.") || text.starts_with("a)") || text.starts_with("1.") || text.starts_with("1)") {
        return Some(McqAnswer::A);
    }
    if text.starts_with("b.") || text.starts_with("b)") || text.starts_with("2.") || text.starts_with("2)") {
        return Some(McqAnswer::B);
    }
    if text.starts_with("c.") || text.starts_with("c)") || text.starts_with("3.") || text.starts_with("3)") {
        return Some(McqAnswer::C);
    }
    if text.starts_with("d.") || text.starts_with("d)") || text.starts_with("4.") || text.starts_with("4)") {
        return Some(McqAnswer::D);
    }
    None
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
        assert_eq!(parse_mcq_answer("A. This is the answer"), Some(McqAnswer::A));
        assert_eq!(parse_mcq_answer("Answer: B"), Some(McqAnswer::B));
        assert_eq!(parse_mcq_answer("The answer is C"), None); // Too strict for now, can relax if needed
        assert_eq!(parse_mcq_answer("1) Paris"), Some(McqAnswer::A));
    }

    #[test]
    fn test_hex_parsing() {
        assert_eq!(parse_hex_color("#FF0000"), 0x000000FF); // Blue=00, Green=00, Red=FF -> 0x000000FF
        assert_eq!(parse_hex_color("#00FF00"), 0x0000FF00);
        assert_eq!(parse_hex_color("#00FFFF"), 0x00FFFF00); // BGR: FF FF 00
        assert_eq!(parse_hex_color("ZZZ"), 0x00000000);
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
