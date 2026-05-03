use once_cell::sync::OnceCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;

#[derive(Debug, Clone)]
pub enum KeyEvent {
    TranslateSelection,
}

static SENDER: OnceCell<mpsc::Sender<KeyEvent>> = OnceCell::new();
static CTRL_DOWN: AtomicBool = AtomicBool::new(false);
static ALT_DOWN: AtomicBool = AtomicBool::new(false);

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

                    match kb.vkCode {
                        0xA2 | 0xA3 => CTRL_DOWN.store(is_keydown, Ordering::SeqCst),
                        0xA4 | 0xA5 => ALT_DOWN.store(is_keydown, Ordering::SeqCst),
                        0x54 if is_keydown => {
                            if CTRL_DOWN.load(Ordering::SeqCst) && ALT_DOWN.load(Ordering::SeqCst) {
                                if let Some(sender) = SENDER.get() {
                                    let _ = sender.send(KeyEvent::TranslateSelection);
                                }
                                return LRESULT(1);
                            }
                        }
                        _ if is_keyup => {}
                        _ => {}
                    }
                }
            }

            CallNextHookEx(None, code, wparam, lparam)
        }

        unsafe {
            let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), None, 0)
                .expect("Failed to set keyboard hook");

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            let _ = UnhookWindowsHookEx(hook);
        }
    });
}
