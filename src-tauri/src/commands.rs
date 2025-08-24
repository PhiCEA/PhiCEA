mod error_log;
mod job;

pub use error_log::*;
pub use job::*;

use super::{AppConfig, Result};
use sqlx::postgres::{PgArguments, PgRow};
use sqlx::PgPool;
use std::ops::Deref;
use tauri::State;
use tokio::sync::RwLock;

#[macro_export]
macro_rules! public_resource {
    ($path:expr) => {{
        #[cfg(debug_assertions)]
        {
            use std::path::{Path, PathBuf};
            let path = Path::new(file!());
            let file_name = path.file_name().unwrap();
            let base_path = path
                .iter()
                .filter(|&part| !part.eq_ignore_ascii_case(file_name))
                .collect::<PathBuf>();
            let mut public_path = base_path.join("../../public");
            public_path.push($path);
            public_path
        }

        #[cfg(not(debug_assertions))]
        {
            use std::path::Path;
            Path::new($path)
        }
    }};
}

#[tauri::command]
pub fn read_config() -> Result<AppConfig> {
    let config = std::fs::read_to_string(public_resource!("config.json"))?;
    Ok(serde_json::from_str(&config)?)
}

/// 配置写入文件
///
/// 建议调用时前端保证 `config` 与原始不同
#[tauri::command]
pub async fn write_config(config: AppConfig, pool: State<'_, RwLock<PgPool>>) -> Result<()> {
    let mut pool = pool.write().await;
    *pool = PgPool::connect(&config.database.url()).await?;

    std::fs::write(
        public_resource!("config.json"),
        serde_json::to_string_pretty(&config)?,
    )?;
    Ok(())
}

/// 通用函数，适用于不同的 SQL 查询和参数
async fn query_as_and_send<'q, T>(
    stmt: &'q str,
    arguments: PgArguments,
    pool: State<'_, RwLock<PgPool>>,
    channel: tauri::ipc::Channel<Vec<u8>>,
) -> Result<()>
where
    T: Send + Unpin + serde::Serialize + for<'r> sqlx::FromRow<'r, PgRow>,
{
    let pool = pool.read().await;
    let data = sqlx::query_as_with::<_, T, PgArguments>(stmt, arguments)
        .fetch_all(pool.deref())
        .await?;
    channel.send(rmp_serde::to_vec(&data)?)?;
    Ok(())
}
