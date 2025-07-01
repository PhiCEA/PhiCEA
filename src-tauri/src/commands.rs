mod error_log;
mod job;

pub use error_log::*;
pub use job::*;

use super::{AppConfig, Result};
use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::Regex;
use sqlx::{
    postgres::{PgArguments, PgRow},
    Executor, PgPool,
};
use std::{fs, ops::Deref, path::PathBuf};
use tauri::State;
use tokio::sync::RwLock;

#[derive(Debug)]
struct JobInfo {
    id: String,
    name: String,
    queue: String,
    n: i32,
    nodes: Vec<String>,
    parameters: Option<String>,
}

lazy_static! {
    static ref TIMESTAMP_PATTERN: Regex = Regex::new(r"(\d{4})-(\d{2})-(\d{2}) (\d{2}):(\d{2}):(\d{2})").unwrap();
    static ref LOG_PATTERN: Regex = Regex::new(r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d{3}).*?l=([\d.e+-]+).*?iter=(\d+).*?err=\{ u=([\d.e+-]+) phi=([\d.e+-]+)").unwrap();
    static ref JOB_INFO_PATTERN: Regex = Regex::new(r"JobInfo\(.*?\bid='([^']*)'.*?\bname='([^']*)'.*?\bqueue='([^']*)'.*?\bn=(\d+).*?\bnodes=\[(.*)\].*\)").unwrap();
    static ref PARAMS_PATTERN: Regex = Regex::new(r"\{.*\}").unwrap();
}

#[tauri::command]
pub async fn import_log(file: PathBuf, pool: State<'_, RwLock<PgPool>>) -> Result<i64> {
    let content = fs::read_to_string(file)?;
    let (job_info, logs) = parse_log_csv(&content)?;

    let insert_job_info = sqlx::query(
        "INSERT INTO job_info (id, name, queue, num_cpu, nodes, parameters) VALUES ($1::bigint, $2, $3, $4, $5, $6::jsonb);",
    )
        .bind(&job_info.id)
        .bind(&job_info.name)
        .bind(&job_info.queue)
        .bind(&job_info.n)
        .bind(&job_info.nodes[..])
        .bind(&job_info.parameters);

    let pool = pool.read().await;
    let mut trans = pool.begin().await?;
    trans.execute(insert_job_info).await?;
    let mut stream = trans.copy_in_raw("COPY error_log (timestamp, load, iter, error_u, error_phi, job_id) FROM STDIN (FORMAT csv);").await?;
    stream.send(logs.as_bytes()).await?;
    stream.finish().await?;
    trans.commit().await?;

    Ok(job_info.id.parse().unwrap())
}

fn parse_log_csv(logs: &str) -> Result<(JobInfo, String)> {
    let (job_info_str, remaining) = logs.split_once('\n').unwrap();
    let (params_str, remaining) = remaining.split_once('\n').unwrap();

    let parameters = PARAMS_PATTERN
        .find(params_str)
        .map(|mat| mat.as_str().to_owned());
    let first_timestamp = TIMESTAMP_PATTERN
        .find(logs)
        .map(|mat| mat.as_str().to_owned())
        .unwrap(); // 一定有匹配

    let job_info = match JOB_INFO_PATTERN.captures(job_info_str) {
        Some(cap) => JobInfo {
            id: cap[1].parse().ok().unwrap(),
            name: cap[2].to_owned(),
            queue: cap[3].to_owned(),
            n: cap[4].parse().ok().unwrap(),
            nodes: cap[5]
                .split(',')
                .map(|s| s.trim_matches(&[' ', '\'']).to_owned())
                .collect(),
            parameters,
        },
        None => JobInfo {
            id: first_timestamp,
            name: String::from("Unknown"),
            queue: String::from("Unknown"),
            n: 0,
            nodes: vec![],
            parameters,
        },
    };

    let logs = remaining
        .par_lines()
        .filter_map(|line| {
            LOG_PATTERN.captures(line).map(|cap| {
                let mut line = cap
                    .iter()
                    .skip(1) // Skip the first capture group (whole match)
                    .flatten() // Flatten the iterator of Option<&str>
                    .map(|m| m.as_str())
                    .collect::<Vec<_>>()
                    .join(",");
                line.push_str(&format!(",{}\n", job_info.id));
                line
            })
        })
        .collect::<String>();

    Ok((job_info, logs))
}

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
