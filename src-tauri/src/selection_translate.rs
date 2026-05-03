use arboard::Clipboard;
use std::thread;
use std::time::{Duration, Instant};

use tauri::async_runtime;

use crate::chinese_detector;
use crate::translate_service::TranslateService;

#[cfg(windows)]
use windows::Win32::System::DataExchange::GetClipboardSequenceNumber;
#[cfg(windows)]
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_KEYUP, VIRTUAL_KEY, VK_CONTROL, VK_INSERT, VK_LCONTROL, VK_LMENU, VK_MENU,
    VK_RCONTROL, VK_RMENU,
};

struct ClipboardBackup {
    text: Option<String>,
}

struct SelectedText {
    text: String,
    backup: ClipboardBackup,
}

pub async fn translate_selection(
    translator: &TranslateService,
    settle_delay_ms: u64,
) -> Result<(), String> {
    let total_start = Instant::now();

    log::info!(
        "Selection workflow started (hotkey, settle_delay_ms={})",
        settle_delay_ms
    );

    let copy_start = Instant::now();
    let selected = async_runtime::spawn_blocking(move || copy_selected_text(settle_delay_ms))
        .await
        .map_err(|err| format!("Failed to run copy workflow: {err}"))??;
    let copy_elapsed = copy_start.elapsed();

    log::info!(
        "Selection workflow copy finished in {}ms (chars={})",
        copy_elapsed.as_millis(),
        selected.text.chars().count()
    );

    if !chinese_detector::contains_chinese(&selected.text) {
        return Err("Selected text does not contain Chinese".to_string());
    }

    let translate_start = Instant::now();
    let translated = translator.translate(selected.text.trim()).await?;
    let translate_elapsed = translate_start.elapsed();
    if translated.trim().is_empty() {
        return Err("Translation result is empty".to_string());
    }

    log::info!(
        "Selection workflow translation finished in {}ms (output_chars={})",
        translate_elapsed.as_millis(),
        translated.chars().count()
    );

    let paste_start = Instant::now();
    async_runtime::spawn_blocking(move || {
        paste_translated_text(selected.backup, translated, settle_delay_ms)
    })
    .await
    .map_err(|err| format!("Failed to run paste workflow: {err}"))??;
    let paste_elapsed = paste_start.elapsed();

    log::info!(
        "Selection workflow paste finished in {}ms",
        paste_elapsed.as_millis()
    );

    log::info!(
        "Selection workflow total={}ms (copy={}ms, translate={}ms, paste={}ms)",
        total_start.elapsed().as_millis(),
        copy_elapsed.as_millis(),
        translate_elapsed.as_millis(),
        paste_elapsed.as_millis()
    );

    Ok(())
}

fn copy_selected_text(settle_delay_ms: u64) -> Result<SelectedText, String> {
    let copy_start = Instant::now();
    let mut clipboard = Clipboard::new().map_err(|e| format!("Clipboard unavailable: {e}"))?;
    let backup = ClipboardBackup {
        text: clipboard.get_text().ok(),
    };

    #[cfg(windows)]
    let sequence_before = unsafe { GetClipboardSequenceNumber() };

    #[cfg(windows)]
    {
        let wait_start = Instant::now();
        wait_for_modifier_keys_release(800)?;
        log::debug!(
            "Selection copy modifiers released in {}ms",
            wait_start.elapsed().as_millis()
        );
    }

    #[cfg(windows)]
    send_ctrl_chord(b'C' as u16);

    let clipboard_wait_start = Instant::now();
    if wait_for_clipboard_change(sequence_before, settle_delay_ms).is_err() {
        log::debug!(
            "Primary Ctrl+C copy did not update clipboard after {}ms, trying Ctrl+Insert fallback",
            clipboard_wait_start.elapsed().as_millis()
        );
        #[cfg(windows)]
        send_ctrl_chord(VK_INSERT.0 as u16);
        let fallback_wait_start = Instant::now();
        wait_for_clipboard_change(sequence_before, settle_delay_ms)?;
        log::debug!(
            "Clipboard updated via fallback copy in {}ms",
            fallback_wait_start.elapsed().as_millis()
        );
    } else {
        log::debug!(
            "Clipboard updated via primary copy in {}ms",
            clipboard_wait_start.elapsed().as_millis()
        );
    }

    let text = clipboard
        .get_text()
        .map_err(|e| format!("No text copied from selection: {e}"))?;
    if text.trim().is_empty() {
        return Err("Copied selection is empty".to_string());
    }

    log::debug!(
        "copy_selected_text total={}ms",
        copy_start.elapsed().as_millis()
    );

    Ok(SelectedText { text, backup })
}

fn paste_translated_text(
    backup: ClipboardBackup,
    translated: String,
    settle_delay_ms: u64,
) -> Result<(), String> {
    let paste_start = Instant::now();
    let mut clipboard = Clipboard::new().map_err(|e| format!("Clipboard unavailable: {e}"))?;
    clipboard
        .set_text(translated)
        .map_err(|e| format!("Failed to set translated text to clipboard: {e}"))?;

    #[cfg(windows)]
    {
        let wait_start = Instant::now();
        wait_for_modifier_keys_release(400)?;
        log::debug!(
            "Selection paste modifiers released in {}ms",
            wait_start.elapsed().as_millis()
        );
    }

    #[cfg(windows)]
    send_ctrl_chord(b'V' as u16);

    thread::sleep(Duration::from_millis(settle_delay_ms));

    if let Some(original_text) = backup.text {
        clipboard
            .set_text(original_text)
            .map_err(|e| format!("Failed to restore original clipboard text: {e}"))?;
    }

    log::debug!(
        "paste_translated_text total={}ms (including restore)",
        paste_start.elapsed().as_millis()
    );

    Ok(())
}

#[cfg(windows)]
fn wait_for_modifier_keys_release(timeout_ms: u64) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    while Instant::now() < deadline {
        let ctrl_down = unsafe { GetAsyncKeyState(VK_CONTROL.0 as i32) } < 0
            || unsafe { GetAsyncKeyState(VK_LCONTROL.0 as i32) } < 0
            || unsafe { GetAsyncKeyState(VK_RCONTROL.0 as i32) } < 0;
        let alt_down = unsafe { GetAsyncKeyState(VK_MENU.0 as i32) } < 0
            || unsafe { GetAsyncKeyState(VK_LMENU.0 as i32) } < 0
            || unsafe { GetAsyncKeyState(VK_RMENU.0 as i32) } < 0;
        if !ctrl_down && !alt_down {
            thread::sleep(Duration::from_millis(30));
            return Ok(());
        }
        thread::sleep(Duration::from_millis(10));
    }
    Err("Hotkey modifiers were not released in time".to_string())
}

fn wait_for_clipboard_change(sequence_before: u32, settle_delay_ms: u64) -> Result<(), String> {
    #[cfg(windows)]
    {
        let deadline = Instant::now() + Duration::from_millis(settle_delay_ms.max(120));
        while Instant::now() < deadline {
            let current = unsafe { GetClipboardSequenceNumber() };
            if current != sequence_before {
                thread::sleep(Duration::from_millis(30));
                return Ok(());
            }
            thread::sleep(Duration::from_millis(20));
        }
        Err("Copy shortcut did not update clipboard".to_string())
    }

    #[cfg(not(windows))]
    {
        let _ = sequence_before;
        thread::sleep(Duration::from_millis(settle_delay_ms));
        Ok(())
    }
}

#[cfg(windows)]
fn send_ctrl_chord(key: u16) {
    unsafe {
        send_key_down(VK_CONTROL.0 as u16);
        send_key_down(key);
        send_key_up(key);
        send_key_up(VK_CONTROL.0 as u16);
    }
}

#[cfg(windows)]
unsafe fn send_key_down(vk: u16) {
    let inputs = [INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: 0,
                dwFlags: KEYBD_EVENT_FLAGS(0),
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }];
    SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
}

#[cfg(windows)]
unsafe fn send_key_up(vk: u16) {
    let inputs = [INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: 0,
                dwFlags: KEYEVENTF_KEYUP,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }];
    SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
}
