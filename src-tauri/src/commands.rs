mod error_log;
mod job;
mod config;

pub use error_log::*;
pub use job::*;
pub use config::*;

use super::Result;
use sqlx::postgres::{PgArguments, PgRow};
use sqlx::PgPool;
use std::ops::Deref;
use tauri::State;
use tokio::sync::RwLock;

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
