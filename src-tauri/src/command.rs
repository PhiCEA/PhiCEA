use super::Result;
use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::Regex;
use sqlx::{Executor, PgPool};
use std::{fs, path::PathBuf};
use tauri::State;
use tokio::sync::Mutex;

#[derive(Debug)]
struct JobInfo {
    id: String,
    name: String,
    queue: String,
    n: i32,
    nodes: Vec<String>,
    parameters: String,
}

lazy_static! {
    static ref TIMESTAMP_PATTERN: Regex = Regex::new(r"(\d{4})-(\d{2})-(\d{2}) (\d{2}):(\d{2}):(\d{2})").unwrap();
    static ref LOG_PATTERN: Regex = Regex::new(r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d{3}).*?l=([\d.e+-]+).*?iter=(\d+).*?err=\{ u=([\d.e+-]+) phi=([\d.e+-]+)").unwrap();
    static ref JOB_INFO_PATTERN: Regex = Regex::new(r"JobInfo\(.*?\bid='([^']*)'.*?\bname='([^']*)'.*?\bqueue='([^']*)'.*?\bn=(\d+).*?\bnodes=\[(.*)\].*\)").unwrap();
    static ref PARAMS_PATTERN: Regex = Regex::new(r"\{.*\}").unwrap();
}

#[tauri::command]
pub async fn import_log(file: PathBuf, pool: State<'_, Mutex<PgPool>>) -> Result<i64> {
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

    let pool = pool.lock().await;
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
        .map_or(String::from(""), |mat| mat.as_str().to_string());
    let first_timestamp = TIMESTAMP_PATTERN
        .captures(logs)
        .map(|cap| {
            cap.iter()
                .skip(1)
                .flatten()
                .map(|m| m.as_str())
                .collect::<String>()
        })
        .unwrap();
    let default_job_info = JobInfo {
        id: first_timestamp,
        name: String::from("Unknown"),
        queue: String::from("Unknown"),
        n: 0,
        nodes: vec![],
        parameters: String::from(""),
    };

    let job_info = JOB_INFO_PATTERN
        .captures(job_info_str)
        .map_or(default_job_info, |cap| JobInfo {
            id: cap[1].parse().ok().unwrap(),
            name: cap[2].to_string(),
            queue: cap[3].to_string(),
            n: cap[4].parse().ok().unwrap(),
            nodes: cap
                .get(5)
                .unwrap()
                .as_str()
                .split(',')
                .map(|s| s.trim_matches(&[' ', '\'']).to_string())
                .collect(),
            parameters,
        });

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
