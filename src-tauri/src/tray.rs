use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, WebviewUrl, WebviewWindowBuilder,
};

/// Distinct from Tauri's default config tray id `"main"` to avoid two systray icons.
const TRAY_ID: &str = "cn2en";

pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    // Config `app.trayIcon` uses id `main`; `remove_tray_by_id` only drops one match but duplicate
    // ids are possible, so loop. Also clear our id in case setup re-runs.
    for id in [TRAY_ID, "main"] {
        while app.tray_by_id(id).is_some() {
            let _ = app.remove_tray_by_id(id);
        }
    }

    let settings_item = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&settings_item, &quit_item])?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(
            app.default_window_icon()
                .expect("missing default icon")
                .clone(),
        )
        .menu(&menu)
        .tooltip("划译 - 已启用")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "settings" => open_settings_window(&app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle().clone();
                tauri::async_runtime::spawn(async move {
                    match crate::toggle_enabled_state().await {
                        Ok(enabled) => {
                            if let Some(tray) = app.tray_by_id(TRAY_ID) {
                                let _ = tray.set_tooltip(Some(if enabled {
                                    "划译 - 已启用"
                                } else {
                                    "划译 - 已暂停"
                                }));
                            }
                        }
                        Err(err) => {
                            log::warn!("Failed to toggle enabled state: {err}");
                        }
                    }
                });
            }
        })
        .build(app)?;

    Ok(())
}

pub fn open_settings_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }

    let _ = WebviewWindowBuilder::new(app, "settings", WebviewUrl::default())
        .title("划译设置")
        .inner_size(420.0, 500.0)
        .center()
        .resizable(false)
        .build();
}
