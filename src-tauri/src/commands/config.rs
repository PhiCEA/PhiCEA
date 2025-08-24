use crate::config::AppConfig;
use super::Result;
use sqlx::PgPool;
use tauri::State;
use tokio::sync::RwLock;

#[tauri::command]
pub fn read_config() -> Result<AppConfig> {
    AppConfig::load()
}

/// 配置写入文件
///
/// 建议调用时前端保证 `config` 与原始不同
#[tauri::command]
pub async fn write_config(config: AppConfig, pool: State<'_, RwLock<PgPool>>) -> Result<()> {
    let mut pool = pool.write().await;
    *pool = PgPool::connect(&config.database.url()).await?;

    config.save()?;
    Ok(())
}