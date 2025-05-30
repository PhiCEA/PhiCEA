mod commands;
mod error;

use commands::Cache;
use sqlx::PgPool;
use tauri::async_runtime;
use tokio::sync::RwLock;

type Result<T> = std::result::Result<T, error::Error>;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct AppConfig {
    database: DatabaseConfig,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct DatabaseConfig {
    user: String,
    password: String,
    host: String,
    port: u16,
    database: String,
}

impl DatabaseConfig {
    fn url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            &self.user, &self.password, &self.host, &self.port, &self.database
        )
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if !std::fs::exists(public_resource!("config.json")).unwrap() {
        std::fs::copy(
            public_resource!("config.example.json"),
            public_resource!("config.json"),
        )
        .unwrap();
    }
    let config = commands::read_config().unwrap();
    let pool =
        async_runtime::block_on(async { PgPool::connect(&config.database.url()).await.unwrap() });

    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(RwLock::new(pool))
        .manage(RwLock::new(Cache::new()))
        .invoke_handler(tauri::generate_handler![
            commands::import_log,
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
