use crate::focused_text::FocusedTextSnapshot;

#[cfg(windows)]
use uiautomation::{
    core::{UIAutomation, UIElement, UITreeWalker},
    patterns::{UILegacyIAccessiblePattern, UITextPattern, UIValuePattern},
};

pub struct UiaService;

impl UiaService {
    pub fn new() -> Self {
        Self
    }

    #[cfg(windows)]
    pub fn read_focused_text(&self) -> Result<FocusedTextSnapshot, String> {
        let automation = UIAutomation::new().map_err(|e| format!("UIA init failed: {e}"))?;
        let focused = automation
            .get_focused_element()
            .map_err(|e| format!("Unable to get focused element: {e}"))?;
        let element = Self::resolve_text_target(&automation, &focused)?;
        let process_id = element
            .get_process_id()
            .map_err(|e| format!("Unable to read process id: {e}"))?;

        if process_id == std::process::id() {
            return Err("focused element belongs to cn2en settings window".to_string());
        }

        let text = Self::read_text_from_element(&element)?;
        if text.trim().is_empty() {
            return Err("focused element text is empty".to_string());
        }

        Ok(FocusedTextSnapshot {
            runtime_id: element
                .get_runtime_id()
                .map_err(|e| format!("Unable to read runtime id: {e}"))?,
            process_id,
            automation_id: element.get_automation_id().unwrap_or_default(),
            class_name: element.get_classname().unwrap_or_default(),
            control_type: format!("{:?}", element.get_control_type().ok()),
            text,
        })
    }

    #[cfg(windows)]
    pub fn replace_focused_text(
        &self,
        snapshot: &FocusedTextSnapshot,
        new_text: &str,
    ) -> Result<(), String> {
        let automation = UIAutomation::new().map_err(|e| format!("UIA init failed: {e}"))?;
        let focused = automation
            .get_focused_element()
            .map_err(|e| format!("Unable to get focused element: {e}"))?;
        let element = Self::resolve_text_target(&automation, &focused)?;

        let runtime_id = element
            .get_runtime_id()
            .map_err(|e| format!("Unable to read runtime id: {e}"))?;
        if runtime_id != snapshot.runtime_id {
            return Err("focused element changed during translation".to_string());
        }

        let current_text = Self::read_text_from_element(&element)?;
        if current_text != snapshot.text {
            return Err("focused element text changed during translation".to_string());
        }

        Self::write_text_to_element(&element, new_text)
    }

    #[cfg(windows)]
    fn read_text_from_element(element: &UIElement) -> Result<String, String> {
        if let Ok(value_pattern) = element.get_pattern::<UIValuePattern>() {
            let value = value_pattern
                .get_value()
                .map_err(|e| format!("Failed to read ValuePattern: {e}"))?;
            if !value.is_empty() {
                return Ok(value);
            }
        }

        if let Ok(legacy_pattern) = element.get_pattern::<UILegacyIAccessiblePattern>() {
            let value = legacy_pattern
                .get_value()
                .map_err(|e| format!("Failed to read LegacyIAccessible value: {e}"))?;
            if !value.is_empty() {
                return Ok(value);
            }
        }

        if let Ok(text_pattern) = element.get_pattern::<UITextPattern>() {
            let range = unsafe { text_pattern.as_ref().DocumentRange() }
                .map_err(|e| format!("Failed to create TextPattern document range: {e}"))?;
            let text = unsafe { range.GetText(-1) }
                .map_err(|e| format!("Failed to read TextPattern text: {e}"))?;
            return Ok(text.to_string());
        }

        Err("focused element does not expose readable text via UIA".to_string())
    }

    #[cfg(windows)]
    fn resolve_text_target(
        automation: &UIAutomation,
        focused: &UIElement,
    ) -> Result<UIElement, String> {
        if Self::read_text_from_element(focused).is_ok() || Self::is_writable_element(focused) {
            return Ok(focused.clone());
        }

        if let Ok(walker) = automation.get_control_view_walker() {
            if let Some(candidate) = Self::search_descendants(&walker, focused) {
                return Ok(candidate);
            }

            if let Some(candidate) = Self::search_ancestors(&walker, focused) {
                return Ok(candidate);
            }
        }

        if let Ok(walker) = automation.get_raw_view_walker() {
            if let Some(candidate) = Self::search_descendants(&walker, focused) {
                return Ok(candidate);
            }

            if let Some(candidate) = Self::search_ancestors(&walker, focused) {
                return Ok(candidate);
            }
        }

        Err("focused element does not expose readable text via UIA".to_string())
    }

    #[cfg(windows)]
    fn search_descendants(walker: &UITreeWalker, root: &UIElement) -> Option<UIElement> {
        let mut queue = std::collections::VecDeque::new();
        if let Some(children) = walker.get_children(root) {
            for child in children {
                queue.push_back((child, 1usize));
            }
        }

        while let Some((element, depth)) = queue.pop_front() {
            if Self::is_candidate_element(&element) {
                return Some(element);
            }

            if depth >= 4 {
                continue;
            }

            if let Some(children) = walker.get_children(&element) {
                for child in children {
                    queue.push_back((child, depth + 1));
                }
            }
        }

        None
    }

    #[cfg(windows)]
    fn search_ancestors(walker: &UITreeWalker, start: &UIElement) -> Option<UIElement> {
        let mut current = start.clone();
        for _ in 0..4 {
            let parent = walker.get_parent(&current).ok()?;
            if Self::is_candidate_element(&parent) {
                return Some(parent);
            }
            if let Some(candidate) = Self::search_descendants(walker, &parent) {
                return Some(candidate);
            }
            current = parent;
        }
        None
    }

    #[cfg(windows)]
    fn is_candidate_element(element: &UIElement) -> bool {
        match Self::read_text_from_element(element) {
            Ok(text) if !text.trim().is_empty() => true,
            _ => Self::is_writable_element(element),
        }
    }

    #[cfg(windows)]
    fn is_writable_element(element: &UIElement) -> bool {
        if let Ok(value_pattern) = element.get_pattern::<UIValuePattern>() {
            if let Ok(is_readonly) = value_pattern.is_readonly() {
                if !is_readonly {
                    return true;
                }
            }
        }

        element.get_pattern::<UILegacyIAccessiblePattern>().is_ok()
    }

    #[cfg(windows)]
    fn write_text_to_element(element: &UIElement, new_text: &str) -> Result<(), String> {
        if let Ok(value_pattern) = element.get_pattern::<UIValuePattern>() {
            let is_readonly = value_pattern
                .is_readonly()
                .map_err(|e| format!("Failed to inspect ValuePattern readonly state: {e}"))?;
            if !is_readonly {
                return value_pattern
                    .set_value(new_text)
                    .map_err(|e| format!("Failed to set ValuePattern value: {e}"));
            }
        }

        if let Ok(legacy_pattern) = element.get_pattern::<UILegacyIAccessiblePattern>() {
            return legacy_pattern
                .set_value(new_text)
                .map_err(|e| format!("Failed to set LegacyIAccessible value: {e}"));
        }

        Err("focused element is readable but not writable via UIA".to_string())
    }

    #[cfg(not(windows))]
    pub fn read_focused_text(&self) -> Result<FocusedTextSnapshot, String> {
        Err("UI Automation is only supported on Windows".to_string())
    }

    #[cfg(not(windows))]
    pub fn replace_focused_text(
        &self,
        _snapshot: &FocusedTextSnapshot,
        _new_text: &str,
    ) -> Result<(), String> {
        Err("UI Automation is only supported on Windows".to_string())
    }
}
