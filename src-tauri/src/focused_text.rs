use crate::chinese_detector::ChineseSegment;

#[derive(Debug, Clone)]
pub struct FocusedTextSnapshot {
    pub runtime_id: Vec<i32>,
    pub process_id: u32,
    pub automation_id: String,
    pub class_name: String,
    pub control_type: String,
    pub text: String,
}

impl FocusedTextSnapshot {
    pub fn replace_tail_segment(
        &self,
        segment: &ChineseSegment,
        replacement: &str,
    ) -> Result<String, String> {
        let chars: Vec<char> = self.text.chars().collect();
        let end = segment.start + segment.char_count;
        if end > chars.len() {
            return Err("segment range exceeds current text".to_string());
        }

        let prefix: String = chars[..segment.start].iter().collect();
        let suffix: String = chars[end..].iter().collect();
        Ok(format!("{prefix}{replacement}{suffix}"))
    }
}
