mod chinese_detector;
mod config;
mod input_monitor;
mod keyboard_hook;
mod selection_translate;
mod translate_service;
mod tray;

use config::AppConfig;
use input_monitor::InputMonitor;
use once_cell::sync::OnceCell;
use std::sync::mpsc;
use tokio::sync::Mutex;
use translate_service::TranslateService;

static TRANSLATOR: OnceCell<TranslateService> = OnceCell::new();
static MONITOR: OnceCell<InputMonitor> = OnceCell::new();
static APP_CONFIG: OnceCell<Mutex<AppConfig>> = OnceCell::new();

fn get_translator() -> &'static TranslateService {
    TRANSLATOR.get().expect("TranslateService not initialized")
}

fn get_monitor() -> &'static InputMonitor {
    MONITOR.get().expect("InputMonitor not initialized")
}

fn get_app_config() -> &'static Mutex<AppConfig> {
    APP_CONFIG.get().expect("AppConfig not initialized")
}

#[tauri::command]
async fn get_config() -> Result<AppConfig, String> {
    Ok(get_app_config().lock().await.clone())
}

#[tauri::command]
async fn save_config(cfg: AppConfig) -> Result<(), String> {
    let cfg = cfg.normalized();
    let translator = get_translator();
    let monitor = get_monitor();

    translator.set_api_key(cfg.api_key.clone()).await;
    translator.set_api_base_url(cfg.api_base_url.clone()).await;
    translator.set_model(cfg.model.clone()).await;
    monitor.set_enabled(cfg.enabled).await;
    *get_app_config().lock().await = cfg.clone();

    config::save_config(&cfg)
}

#[tauri::command]
async fn toggle_enabled() -> Result<bool, String> {
    toggle_enabled_state().await
}

pub(crate) async fn toggle_enabled_state() -> Result<bool, String> {
    let monitor = get_monitor();
    let current = monitor.is_enabled().await;
    let new_state = !current;
    monitor.set_enabled(new_state).await;

    let mut cfg = get_app_config().lock().await.clone();
    cfg.enabled = new_state;
    *get_app_config().lock().await = cfg.clone();
    config::save_config(&cfg)?;

    Ok(new_state)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let cfg = config::load_config();
    let translator = TranslateService::new();
    let monitor = InputMonitor::new(translator.clone());

    TRANSLATOR.set(translator.clone()).ok();
    MONITOR.set(monitor).ok();
    APP_CONFIG.set(Mutex::new(cfg.clone())).ok();

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tray::open_settings_window(app);
        }))
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            toggle_enabled
        ])
        .setup(move |app| {
            let enabled = cfg.enabled;
            let cfg_for_runtime = cfg.clone();
            let _ = config::init_config_file_if_missing();
            let t = translator.clone();
            tauri::async_runtime::spawn(async move {
                t.set_api_key(cfg_for_runtime.api_key).await;
                t.set_api_base_url(cfg_for_runtime.api_base_url).await;
                t.set_model(cfg_for_runtime.model).await;
            });

            let monitor = get_monitor();
            tauri::async_runtime::block_on(async {
                monitor.set_enabled(enabled).await;
            });

            // Start keyboard hook and input monitor
            let (sender, receiver) = mpsc::channel();
            keyboard_hook::start_hook(sender);
            monitor.start(receiver);
            tray::setup_tray(app.handle())?;

            log::info!("cn2en started. Press Ctrl+Alt+T to translate selected text.");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
