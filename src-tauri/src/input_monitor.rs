use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::Arc;

use tauri::async_runtime;
use tokio::sync::Mutex;

use crate::keyboard_hook::KeyEvent;
use crate::selection_translate;
use crate::translate_service::TranslateService;

const SETTLE_DELAY_MS: u64 = 80;

pub struct InputMonitor {
    enabled: Arc<Mutex<bool>>,
    busy: Arc<AtomicU64>,
    translator: TranslateService,
}

impl InputMonitor {
    pub fn new(translator: TranslateService) -> Self {
        Self {
            enabled: Arc::new(Mutex::new(true)),
            busy: Arc::new(AtomicU64::new(0)),
            translator,
        }
    }

    pub async fn set_enabled(&self, val: bool) {
        *self.enabled.lock().await = val;
    }

    pub async fn is_enabled(&self) -> bool {
        *self.enabled.lock().await
    }

    pub fn start(&self, receiver: mpsc::Receiver<KeyEvent>) {
        let enabled = self.enabled.clone();
        let busy = self.busy.clone();
        let translator = self.translator.clone();

        async_runtime::spawn(async move {
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<KeyEvent>();

            std::thread::spawn(move || {
                while let Ok(event) = receiver.recv() {
                    if tx.send(event).is_err() {
                        break;
                    }
                }
            });

            while let Some(event) = rx.recv().await {
                match event {
                    KeyEvent::TranslateSelection => {
                        if !*enabled.lock().await {
                            continue;
                        }
                        if busy
                            .compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst)
                            .is_err()
                        {
                            log::debug!("Translation shortcut ignored because a task is running");
                            continue;
                        }

                        log::info!(
                            "Selection shortcut triggered (settle_delay_ms={})",
                            SETTLE_DELAY_MS
                        );
                        let task_translator = translator.clone();
                        let task_busy = busy.clone();

                        async_runtime::spawn(async move {
                            let result = selection_translate::translate_selection(
                                &task_translator,
                                SETTLE_DELAY_MS,
                            )
                            .await;
                            match result {
                                Ok(()) => {
                                    log::info!("Selection translated and replaced successfully");
                                }
                                Err(err) => {
                                    log::warn!("Selection translation skipped/failed: {err}");
                                }
                            }
                            task_busy.store(0, Ordering::SeqCst);
                        });
                    }
                }
            }
        });
    }
}
