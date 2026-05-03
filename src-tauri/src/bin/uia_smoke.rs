#[path = "../chinese_detector.rs"]
mod chinese_detector;
#[path = "../focused_text.rs"]
mod focused_text;
#[path = "../uia_service.rs"]
mod uia_service;

use std::thread;
use std::time::Duration;

use uia_service::UiaService;

fn main() {
    let replacement = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "hello".to_string());
    let delay_ms = std::env::args()
        .nth(2)
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(2000);

    eprintln!("Waiting {delay_ms}ms before probing focused element...");
    thread::sleep(Duration::from_millis(delay_ms));

    let service = UiaService::new();
    let snapshot = match service.read_focused_text() {
        Ok(snapshot) => snapshot,
        Err(err) => {
            eprintln!("UIA read failed: {err}");
            std::process::exit(1);
        }
    };

    eprintln!(
        "Focused element: class={} control={} text={:?}",
        snapshot.class_name, snapshot.control_type, snapshot.text
    );

    let Some(segment) = chinese_detector::extract_tail_chinese_segment(&snapshot.text) else {
        eprintln!("No tail Chinese segment found.");
        std::process::exit(2);
    };

    let new_text = match snapshot.replace_tail_segment(&segment, &replacement) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("Failed to construct replacement text: {err}");
            std::process::exit(3);
        }
    };

    match service.replace_focused_text(&snapshot, &new_text) {
        Ok(()) => {
            eprintln!(
                "UIA replacement succeeded: {:?} -> {:?}",
                segment.text, replacement
            );
        }
        Err(err) => {
            eprintln!("UIA replacement failed: {err}");
            std::process::exit(4);
        }
    }
}
