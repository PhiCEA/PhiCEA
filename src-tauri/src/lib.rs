mod commands;
mod config;
mod error;

use commands::Cache;
use config::AppConfig;
use sqlx::PgPool;
use tauri::async_runtime;
use tokio::sync::RwLock;

type Result<T> = std::result::Result<T, error::Error>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = AppConfig::load().expect("Failed to load config");
    let pool = async_runtime::block_on(async {
        PgPool::connect(&config.database.url())
            .await
            .expect("Failed to connect to database")
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(RwLock::new(pool))
        .manage(RwLock::new(Cache::new()))
        .invoke_handler(tauri::generate_handler![
            commands::import_error_log,
            commands::read_config,
            commands::write_config,
            commands::get_total_time,
            commands::get_job_list,
            commands::get_error_log,
            commands::clear_error_log_cache,
            commands::find_job,
            commands::remove_job,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
