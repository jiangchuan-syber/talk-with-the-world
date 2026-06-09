#[path = "../config.rs"]
mod config;
#[path = "../translate_service.rs"]
mod translate_service;

use translate_service::TranslateService;

#[tokio::main]
async fn main() {
    let input = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "你好，世界".to_string());

    let cfg = config::load_config();
    if cfg.api_key.trim().is_empty() {
        eprintln!("No API key configured.");
        std::process::exit(1);
    }

    let service = TranslateService::new();
    service.set_api_key(cfg.api_key).await;
    service.set_model(cfg.model).await;

    match service.translate(&input).await {
        Ok(output) => {
            println!("{output}");
        }
        Err(err) => {
            eprintln!("Translation failed: {err}");
            std::process::exit(2);
        }
    }
}
