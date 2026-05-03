#[cfg(windows)]
use std::collections::VecDeque;
use std::thread;
use std::time::Duration;

#[cfg(windows)]
use uiautomation::{
    core::{UIAutomation, UIElement, UITreeWalker},
    patterns::{UILegacyIAccessiblePattern, UITextPattern, UIValuePattern},
};

#[cfg(windows)]
fn main() {
    let delay_ms = std::env::args()
        .nth(1)
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(8000);

    eprintln!("Waiting {delay_ms}ms. Focus the target input now...");
    thread::sleep(Duration::from_millis(delay_ms));

    let automation = match UIAutomation::new() {
        Ok(v) => v,
        Err(err) => {
            eprintln!("UIA init failed: {err}");
            std::process::exit(1);
        }
    };

    let focused = match automation.get_focused_element() {
        Ok(v) => v,
        Err(err) => {
            eprintln!("Get focused element failed: {err}");
            std::process::exit(2);
        }
    };

    eprintln!("== Focused ==");
    print_summary(&focused, 0);

    if let Ok(control_walker) = automation.get_control_view_walker() {
        eprintln!("\n== Ancestors (control view) ==");
        dump_ancestors(&control_walker, &focused);
        eprintln!("\n== Descendants (control view, depth<=4) ==");
        dump_descendants(&control_walker, &focused, 4);
    }

    if let Ok(raw_walker) = automation.get_raw_view_walker() {
        eprintln!("\n== Descendants (raw view, depth<=4) ==");
        dump_descendants(&raw_walker, &focused, 4);
    }
}

#[cfg(not(windows))]
fn main() {
    eprintln!("uia_probe only supports Windows");
}

#[cfg(windows)]
fn dump_ancestors(walker: &UITreeWalker, start: &UIElement) {
    let mut current = start.clone();
    for depth in 1..=5 {
        let Ok(parent) = walker.get_parent(&current) else {
            break;
        };
        print_summary(&parent, depth);
        current = parent;
    }
}

#[cfg(windows)]
fn dump_descendants(walker: &UITreeWalker, root: &UIElement, max_depth: usize) {
    let mut queue = VecDeque::new();
    if let Some(children) = walker.get_children(root) {
        for child in children {
            queue.push_back((child, 1usize));
        }
    }

    while let Some((element, depth)) = queue.pop_front() {
        print_summary(&element, depth);
        if depth >= max_depth {
            continue;
        }
        if let Some(children) = walker.get_children(&element) {
            for child in children {
                queue.push_back((child, depth + 1));
            }
        }
    }
}

#[cfg(windows)]
fn print_summary(element: &UIElement, depth: usize) {
    let indent = "  ".repeat(depth);
    let class_name = element.get_classname().unwrap_or_default();
    let name = element.get_name().unwrap_or_default();
    let automation_id = element.get_automation_id().unwrap_or_default();
    let control_type = format!("{:?}", element.get_control_type().ok());
    let process_id = element.get_process_id().unwrap_or_default();

    let value_text = element
        .get_pattern::<UIValuePattern>()
        .ok()
        .and_then(|p| p.get_value().ok());
    let value_writable = element
        .get_pattern::<UIValuePattern>()
        .ok()
        .and_then(|p| p.is_readonly().ok())
        .map(|v| !v)
        .unwrap_or(false);
    let legacy_text = element
        .get_pattern::<UILegacyIAccessiblePattern>()
        .ok()
        .and_then(|p| p.get_value().ok());
    let legacy_writable = element.get_pattern::<UILegacyIAccessiblePattern>().is_ok();
    let text_pattern_text = element
        .get_pattern::<UITextPattern>()
        .ok()
        .and_then(|p| unsafe { p.as_ref().DocumentRange() }.ok())
        .and_then(|r| unsafe { r.GetText(-1) }.ok())
        .map(|s| s.to_string());

    eprintln!(
        "{indent}- class={class_name:?} name={name:?} aid={automation_id:?} ctrl={control_type} pid={process_id} value_write={value_writable} legacy_write={legacy_writable} value={value_text:?} legacy={legacy_text:?} text={text_pattern_text:?}"
    );
}
