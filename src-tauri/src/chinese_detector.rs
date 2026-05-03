pub fn contains_chinese(text: &str) -> bool {
    text.chars().any(is_cjk)
}

#[allow(dead_code)]
pub fn extract_tail_chinese_segment(text: &str) -> Option<ChineseSegment> {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return None;
    }

    let mut end = chars.len();
    while end > 0 && chars[end - 1].is_whitespace() {
        end -= 1;
    }

    if end == 0 || !is_cjk(chars[end - 1]) {
        return None;
    }

    let mut start = end - 1;
    while start > 0 && is_cjk(chars[start - 1]) {
        start -= 1;
    }

    let segment: String = chars[start..end].iter().collect();
    Some(ChineseSegment {
        text: segment,
        start,
        char_count: end - start,
    })
}

fn is_cjk(c: char) -> bool {
    matches!(c,
        '\u{4e00}'..='\u{9fff}' |   // CJK Unified Ideographs
        '\u{3400}'..='\u{4dbf}' |   // CJK Extension A
        '\u{f900}'..='\u{faff}' |   // CJK Compatibility Ideographs
        '\u{3000}'..='\u{303f}' |   // CJK Symbols and Punctuation
        '\u{ff00}'..='\u{ffef}'     // Fullwidth Forms
    )
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChineseSegment {
    pub text: String,
    pub start: usize,
    pub char_count: usize,
}
