use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::Arc;

use tauri::async_runtime;
use tokio::sync::Mutex;

use crate::keyboard_hook::KeyEvent;
use crate::selection_translate;
use crate::translate_service::TranslateService;

pub struct InputMonitor {
    operation_delay_ms: Arc<Mutex<u64>>,
    enabled: Arc<Mutex<bool>>,
    busy: Arc<AtomicU64>,
    translator: TranslateService,
}

impl InputMonitor {
    pub fn new(translator: TranslateService) -> Self {
        Self {
            operation_delay_ms: Arc::new(Mutex::new(180)),
            enabled: Arc::new(Mutex::new(true)),
            busy: Arc::new(AtomicU64::new(0)),
            translator,
        }
    }

    pub async fn set_delay(&self, ms: u64) {
        *self.operation_delay_ms.lock().await = ms.clamp(80, 800);
    }

    pub async fn set_enabled(&self, val: bool) {
        *self.enabled.lock().await = val;
    }

    pub async fn is_enabled(&self) -> bool {
        *self.enabled.lock().await
    }

    pub fn start(&self, receiver: mpsc::Receiver<KeyEvent>) {
        let operation_delay_ms = self.operation_delay_ms.clone();
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

                        let settle_ms = (*operation_delay_ms.lock().await).clamp(80, 800);
                        log::info!(
                            "Selection shortcut triggered (settle_delay_ms={})",
                            settle_ms
                        );
                        let task_translator = translator.clone();
                        let task_busy = busy.clone();

                        async_runtime::spawn(async move {
                            let result = selection_translate::translate_selection(
                                &task_translator,
                                settle_ms,
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
