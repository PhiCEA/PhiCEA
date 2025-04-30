mod command;
mod error;

use sqlx::PgPool;
use tauri::async_runtime;
use tokio::sync::Mutex;

type Result<T> = std::result::Result<T, error::Error>;

// struct ErrorLog {
//     time: NaiveDateTime,
//     load: f64,
//     iter: i32,
//     error_u: f64,
//     error_phi: f64,
// }

#[derive(serde::Deserialize)]
struct AppConfig {
    database: DatabaseConfig,
}

#[derive(serde::Deserialize)]
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

fn load_config() -> Result<AppConfig> {
    let config_file_path = if cfg!(debug_assertions) {
        // vite dev server serves config.json from /public
        "../public/config.json"
    } else {
        "config.json"
    };
    let config_file = std::fs::read_to_string(config_file_path)?;
    Ok(serde_json::from_str(&config_file)?)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = load_config().unwrap();
    let pool =
        async_runtime::block_on(async { PgPool::connect(&config.database.url()).await.unwrap() });

    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(pool))
        .invoke_handler(tauri::generate_handler![command::import_log])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
