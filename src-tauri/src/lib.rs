mod command;
mod error;

use sqlx::PgPool;
use tauri::async_runtime;
use tokio::sync::RwLock;

type Result<T> = std::result::Result<T, error::Error>;

// struct ErrorLog {
//     time: NaiveDateTime,
//     load: f64,
//     iter: i32,
//     error_u: f64,
//     error_phi: f64,
// }

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
    let config = command::read_config().unwrap();
    let pool =
        async_runtime::block_on(async { PgPool::connect(&config.database.url()).await.unwrap() });

    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(RwLock::new(pool))
        .invoke_handler(tauri::generate_handler![
            command::import_log,
            command::read_config,
            command::write_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
