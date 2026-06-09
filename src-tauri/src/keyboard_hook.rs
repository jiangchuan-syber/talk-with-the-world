use once_cell::sync::OnceCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum KeyEvent {
    TranslateSelection,
}

static SENDER: OnceCell<mpsc::Sender<KeyEvent>> = OnceCell::new();
static CTRL_DOWN: AtomicBool = AtomicBool::new(false);
static SHIFT_DOWN: AtomicBool = AtomicBool::new(false);

#[cfg(windows)]
fn hotkey_modifiers_down() -> bool {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, VK_CONTROL, VK_LCONTROL, VK_LSHIFT, VK_RCONTROL, VK_RSHIFT, VK_SHIFT,
    };

    unsafe {
        let ctrl_down = GetAsyncKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000 != 0
            || GetAsyncKeyState(VK_LCONTROL.0 as i32) as u16 & 0x8000 != 0
            || GetAsyncKeyState(VK_RCONTROL.0 as i32) as u16 & 0x8000 != 0;
        let shift_down = GetAsyncKeyState(VK_SHIFT.0 as i32) as u16 & 0x8000 != 0
            || GetAsyncKeyState(VK_LSHIFT.0 as i32) as u16 & 0x8000 != 0
            || GetAsyncKeyState(VK_RSHIFT.0 as i32) as u16 & 0x8000 != 0;
        ctrl_down && shift_down
    }
}

#[cfg(windows)]
fn sync_modifier_state(vk_code: u32, is_keydown: bool) {
    match vk_code {
        0xA2 | 0xA3 | 0x11 => CTRL_DOWN.store(is_keydown, Ordering::SeqCst),
        0xA0 | 0xA1 | 0x10 => SHIFT_DOWN.store(is_keydown, Ordering::SeqCst),
        _ => {}
    }
}

#[cfg(windows)]
pub fn start_hook(sender: mpsc::Sender<KeyEvent>) {
    use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::*;

    thread::spawn(move || {
        let _ = SENDER.set(sender);

        unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
            if code >= 0 {
                let kb = *(lparam.0 as *const KBDLLHOOKSTRUCT);
                if kb.flags.0 & LLKHF_INJECTED.0 == 0 {
                    let is_keydown =
                        wparam.0 == WM_KEYDOWN as usize || wparam.0 == WM_SYSKEYDOWN as usize;
                    let is_keyup =
                        wparam.0 == WM_KEYUP as usize || wparam.0 == WM_SYSKEYUP as usize;

                    sync_modifier_state(kb.vkCode, is_keydown);

                    if kb.vkCode == 0x41 && is_keydown {
                        let tracked = CTRL_DOWN.load(Ordering::SeqCst)
                            && SHIFT_DOWN.load(Ordering::SeqCst);
                        if tracked || hotkey_modifiers_down() {
                            if let Some(sender) = SENDER.get() {
                                let _ = sender.send(KeyEvent::TranslateSelection);
                            }
                            return LRESULT(1);
                        }
                    }
                    let _ = is_keyup;
                }
            }

            CallNextHookEx(None, code, wparam, lparam)
        }

        loop {
            unsafe {
                let hook = match SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), None, 0) {
                    Ok(h) => h,
                    Err(err) => {
                        log::error!("Failed to set keyboard hook: {err:?}, retrying in 1s");
                        thread::sleep(Duration::from_secs(1));
                        continue;
                    }
                };

                log::info!("Global keyboard hook installed (Ctrl+Shift+A)");

                let mut msg = MSG::default();
                while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }

                let _ = UnhookWindowsHookEx(hook);
                log::warn!("Keyboard hook message loop exited, reinstalling in 500ms");
            }
            thread::sleep(Duration::from_millis(500));
        }
    });
}
