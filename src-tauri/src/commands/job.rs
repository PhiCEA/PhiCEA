use std::ops::Deref;

use crate::error::Error;

use super::{query_as_and_send, Result};
use sqlx::postgres::PgArguments;
use sqlx::PgPool;
use tauri::State;
use tokio::sync::RwLock;

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct JobInfo {
    id: i64,
    name: String,
    queue: String,
    num_cpu: i32,
    parameters: Option<serde_json::Value>,
}

#[tauri::command]
pub async fn get_job_list(
    channel: tauri::ipc::Channel<Vec<u8>>,
    pool: State<'_, RwLock<PgPool>>,
) -> Result<()> {
    let stmt = r#"
        SELECT 
            id, name, queue, num_cpu, parameters 
        FROM job_info;"#;
    let args = PgArguments::default();
    query_as_and_send::<JobInfo>(stmt, args, pool, channel).await
}

#[tauri::command]
pub async fn find_job(job_id: i64, pool: State<'_, RwLock<PgPool>>) -> Result<JobInfo> {
    let stmt = r#"
        SELECT id, name, queue, num_cpu, parameters 
        FROM job_info 
        WHERE id = $1;"#;
    let pool = pool.read().await;
    sqlx::query_as::<_, JobInfo>(stmt)
        .bind(job_id)
        .fetch_one(pool.deref())
        .await
        .map_err(Error::Sqlx)
}

#[tauri::command]
pub async fn remove_job(job_id: i64, pool: State<'_, RwLock<PgPool>>) -> Result<()> {
    let stmt = r#"
        DELETE FROM job_info 
        WHERE id = $1;"#;
    let pool = pool.read().await;
    match sqlx::query(stmt).bind(job_id).execute(pool.deref()).await {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::Sqlx(e)),
    }
}
