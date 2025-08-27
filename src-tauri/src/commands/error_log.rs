use crate::error::Error;

use super::Result;
use ahash::AHashMap;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use lazy_static::lazy_static;
use rayon::iter::ParallelIterator;
use rayon::str::ParallelString;
use regex::Regex;
use sqlx::{Executor, PgPool, Row};
use std::collections::VecDeque;
use std::fs;
use std::io::{Read, Write};
use std::ops::Deref;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};
use tokio::sync::RwLock;

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub(crate) struct ErrorLogEntry {
    iters: i32,
    load: f64,
    error_u: f64,
    error_phi: f64,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub(crate) struct ErrorLogSummary {
    load: f64,
    iters: i32,
    cost: Option<f64>,
}

/// 缓存结构体，用于存储压缩后的日志数据。
///
/// 该缓存使用 LRU（最近最少使用）策略来管理缓存项，
/// 并使用 Gzip 压缩/解压数据以减少内存占用。
pub struct Cache {
    map: AHashMap<i64, Vec<u8>>,
    queue: VecDeque<i64>,
    max_size: usize,
}

impl Cache {
    pub fn new() -> Self {
        const MAX_SIZE: usize = 8;
        Self {
            queue: VecDeque::with_capacity(MAX_SIZE),
            max_size: MAX_SIZE,
            map: AHashMap::new(),
        }
    }

    pub fn has(&self, key: i64) -> bool {
        self.map.contains_key(&key)
    }

    pub fn get(&self, key: i64) -> Option<Vec<u8>> {
        self.map.get(&key).and_then(|item| {
            let mut decoder = GzDecoder::new(item.as_slice());
            let mut buffer = Vec::new();
            match decoder.read_to_end(&mut buffer) {
                Ok(_) => Some(buffer),
                Err(_) => None,
            }
        })
    }

    pub fn set(&mut self, key: i64, value: &[u8]) -> Result<()> {
        if self.has(key) {
            return Ok(());
        }
        if self.map.len() >= self.max_size {
            let outdated_key = self.queue.pop_front().unwrap();
            self.map.remove(&outdated_key);
        }
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(value)?;
        self.map.insert(key, encoder.finish()?);
        self.queue.push_back(key);

        Ok(())
    }

    pub fn remove(&mut self, key: i64) {
        if self.has(key) {
            self.map.remove(&key);
            self.queue.retain(|x| *x != key);
        }
    }

    pub fn clear(&mut self) {
        self.map.clear();
        self.queue.clear();
    }
}

lazy_static! {
    static ref TIMESTAMP_PATTERN: Regex = Regex::new(r"(\d{4})-(\d{2})-(\d{2}) (\d{2}):(\d{2}):(\d{2})").unwrap();
    static ref LOG_PATTERN: Regex = Regex::new(r"(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d{3}).*?l=([\d.e+-]+).*?iter=(\d+).*?err=\{ u=([\d.e+-]+) phi=([\d.e+-]+)").unwrap();
    static ref JOB_INFO_PATTERN: Regex = Regex::new(r"JobInfo\(.*?\bid='([^']*)'.*?\bname='([^']*)'.*?\bqueue='([^']*)'.*?\bn=(\d+).*?\bnodes=\[(.*)\].*\)").unwrap();
    static ref PARAMS_PATTERN: Regex = Regex::new(r"\{.*\}").unwrap();
}

struct LogParser;

impl LogParser {
    /// 该函数用于解析包含作业信息和误差日志的文本内容，提取出结构化的作业信息
    /// 并将日志数据转换为CSV格式以便后续导入数据库。
    ///
    /// # 日志格式要求
    ///
    /// 输入的日志内容应遵循特定格式：
    /// 1. 第一行为作业信息（[`JobInfo`]），格式如：`JobInfo(id='...', name='...', queue='...', n=..., nodes=[...])`
    /// 2. 第二行为参数信息（JSON格式字符串，可选）
    /// 3. 后续行为具体的日志条目，每行包含时间戳、加载步、单步迭代次数、迭代误差等信息
    ///
    /// # 返回值
    ///
    /// 返回一个Result元组：
    /// * `Ok((JobInfo, String))`: 成功时返回解析出的作业详细信息结构体和转换后的CSV格式日志字符串
    /// * `Err(...)`: 解析失败时返回错误信息
    ///
    /// CSV格式的日志数据包含以下列：
    /// timestamp, load, iter, error_u, error_phi, job_id
    ///
    /// # 示例
    ///
    /// ```text
    /// JobInfo(id='666666', name='test_job', queue='default', n=4, nodes=['node1', 'node2'])
    /// {param1: value1, param2: value2}
    /// 2023-01-01 10:00:00.000 ... l=1.5 ... iter=1 ... err={ u=0.1 phi=0.2 }
    /// 2023-01-01 10:01:00.000 ... l=1.2 ... iter=2 ... err={ u=0.05 phi=0.15 }
    /// ```
    ///
    /// 将被解析为：
    ///
    /// ```text
    /// JobInfo { id: "666666", name: "test_job", queue: "default", n: 4, nodes: ["node1", "node2"], parameters: Some("{param1: value1, param2: value2}") }
    /// ```
    ///
    /// 和CSV数据：
    ///
    /// ```csv
    /// 2023-01-01 10:00:00.000,1.5,1,0.1,0.2,666666
    /// 2023-01-01 10:01:00.000,1.2,2,0.05,0.15,666666
    /// ```
    fn parse_to_csv(logs: &str) -> Result<(JobInfo, String)> {
        let (job_info_str, remaining) = logs
            .split_once('\n')
            .ok_or(Error::LogFormat(String::from("格式错误：第一行附近")))?;
        let (params_str, remaining) = remaining
            .split_once('\n')
            .ok_or(Error::LogFormat(String::from("格式错误：第二行附近")))?;

        // 解析参数
        let parameters = PARAMS_PATTERN
            .find(params_str)
            .map(|mat| mat.as_str().to_owned());

        // 解析作业信息
        let job_info = JOB_INFO_PATTERN
            .captures(job_info_str)
            .map(|cap| JobInfo {
                id: cap[1].parse().ok().unwrap(),
                name: cap[2].to_owned(),
                queue: cap[3].to_owned(),
                n: cap[4].parse().ok().unwrap(),
                nodes: cap[5]
                    .split(',')
                    .map(|s| s.trim_matches(&[' ', '\'']).to_owned())
                    .collect(),
                parameters,
            })
            .ok_or(Error::LogFormat(String::from("Cannot parse job info")))?;

        // 根据误差日志第一行提取
        let indices = LOG_PATTERN
            .captures(logs)
            .map(|cap| {
                cap.iter()
                    .skip(1)
                    .flatten()
                    .map(|m| m.range())
                    .collect::<Vec<_>>()
            })
            .ok_or(Error::LogFormat(String::from("Cannot build index")))?;

        // 利用索引提取字段，构建CSV
        let csv = remaining
            .par_lines()
            .filter_map(|line| {
                LOG_PATTERN.is_match(line).then(|| {
                    let mut row = indices
                        .iter()
                        .cloned()
                        .map(|range| &line[range])
                        .collect::<Vec<_>>()
                        .join(",");
                    row.push_str(&format!(",{}\n", job_info.id));
                    row
                })
            })
            .collect::<String>();

        Ok((job_info, csv))
    }
}

#[derive(Debug)]
struct JobInfo {
    id: String,
    name: String,
    queue: String,
    n: i32,
    nodes: Vec<String>,
    parameters: Option<String>,
}

#[tauri::command]
pub async fn import_error_log(file: PathBuf, pool: State<'_, RwLock<PgPool>>) -> Result<i64> {
    let content = fs::read_to_string(file)?;
    let (job_info, logs) = LogParser::parse_to_csv(&content)?;

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

    job_info
        .id
        .parse()
        .map_err(|_| Error::LogFormat(String::from("job id is not a number")))
}

#[tauri::command]
pub async fn get_error_log(
    job_id: i64,
    channel: tauri::ipc::Channel<Vec<u8>>,
    cache: State<'_, RwLock<Cache>>,
    pool: State<'_, RwLock<PgPool>>,
    app: AppHandle,
) -> Result<()> {
    // 先查询缓存
    if let Some(ceched_rmp) = cache.read().await.get(job_id) {
        channel.send(ceched_rmp)?;
        return Ok(());
    }

    // 没有缓存，再查询数据库
    let stmt_entries = r#"
            SELECT 
                (ROW_NUMBER() OVER (ORDER BY timestamp))::INTEGER as iters, load, error_u, error_phi 
            FROM error_log 
            WHERE job_id = $1 
            ORDER BY timestamp;"#;
    let stmt_summary = r#"
            SELECT 
                load, iters, extract(EPOCH from lag(timestamp, -1) over (order by load) - timestamp)::DOUBLE PRECISION as cost 
            FROM error_log_summary 
            WHERE job_id = $1;"#;

    let pool = pool.read().await;
    let ret = tokio::try_join!(
        sqlx::query_as::<_, ErrorLogSummary>(stmt_summary)
            .bind(job_id)
            .fetch_all(pool.deref()),
        sqlx::query_as::<_, ErrorLogEntry>(stmt_entries)
            .bind(job_id)
            .fetch_all(pool.deref()),
    );
    match ret {
        Ok(data) => {
            let rmp = rmp_serde::to_vec(&data)?;
            let cloned = rmp.clone();
            tokio::spawn(async move {
                let cache = app.state::<RwLock<Cache>>();
                cache.write().await.set(job_id, &cloned).unwrap();
            });

            channel.send(rmp).map_err(Error::Tauri)
        }
        Err(err) => return Err(err.into()),
    }
}

/// Get total solving time in seconds
#[tauri::command]
pub async fn get_total_time(job_id: i64, pool: State<'_, RwLock<PgPool>>) -> Result<f64> {
    let stmt = r#"
        SELECT 
            extract(EPOCH from max(timestamp) - min(timestamp))::DOUBLE PRECISION as total 
        FROM error_log 
        WHERE job_id = $1;"#;

    let pool = pool.read().await;

    match sqlx::query(stmt).bind(job_id).fetch_one(pool.deref()).await {
        Ok(row) => Ok(row.get(0)),
        Err(e) => Err(e.into()),
    }
}

#[tauri::command]
pub async fn clear_error_log_cache(cache: State<'_, RwLock<Cache>>) -> Result<()> {
    cache.write().await.clear();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_new() {
        let cache = Cache::new();
        assert_eq!(cache.map.len(), 0);
        assert_eq!(cache.queue.len(), 0);
        assert_eq!(cache.max_size, 8);
    }

    #[test]
    fn test_cache_set_and_get() {
        let mut cache = Cache::new();
        let key = 1i64;
        let value = b"test data";

        // 设置缓存项
        assert!(cache.set(key, value).is_ok());

        // 验证缓存项存在
        assert!(cache.has(key));

        // 获取缓存项
        let retrieved = cache.get(key);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), value);
    }

    #[test]
    fn test_cache_has_not_exists() {
        let cache = Cache::new();
        assert!(!cache.has(1));
    }

    #[test]
    fn test_cache_get_not_exists() {
        let cache = Cache::new();
        assert!(cache.get(1).is_none());
    }

    #[test]
    fn test_cache_set_duplicate() {
        let mut cache = Cache::new();
        let key = 1i64;
        let value1 = b"first data";
        let value2 = b"second data";

        // 设置第一个值
        assert!(cache.set(key, value1).is_ok());

        // 尝试设置相同的键（应该被忽略）
        assert!(cache.set(key, value2).is_ok());

        // 验证值仍然是第一个
        let retrieved = cache.get(key);
        assert_eq!(retrieved.unwrap(), value1);
    }

    #[test]
    fn test_cache_lru_eviction() {
        let mut cache = Cache::new();

        // 填满缓存（最大容量为8）
        for i in 0..8 {
            let value = format!("data{}", i).into_bytes();
            assert!(cache.set(i, &value).is_ok());
        }

        // 验证所有项都存在
        for i in 0..8 {
            assert!(cache.has(i));
        }

        // 添加第9个项，应该触发LRU驱逐（键0被移除）
        let value = b"data8";
        assert!(cache.set(8, value).is_ok());

        // 验证键0已被移除，其他项仍然存在
        assert!(!cache.has(0));
        for i in 1..=8 {
            assert!(cache.has(i));
        }
    }

    #[test]
    fn test_cache_remove() {
        let mut cache = Cache::new();
        let key = 1i64;
        let value = b"test data";

        // 设置并确认存在
        assert!(cache.set(key, value).is_ok());
        assert!(cache.has(key));

        // 删除项
        cache.remove(key);

        // 验证项已被删除
        assert!(!cache.has(key));
        assert!(cache.get(key).is_none());
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = Cache::new();

        // 添加几个项
        for i in 0..3 {
            let value = format!("data{}", i).into_bytes();
            assert!(cache.set(i, &value).is_ok());
        }

        // 验证项存在
        assert_eq!(cache.map.len(), 3);
        assert_eq!(cache.queue.len(), 3);

        // 清空缓存
        cache.clear();

        // 验证缓存已清空
        assert_eq!(cache.map.len(), 0);
        assert_eq!(cache.queue.len(), 0);
    }

    #[test]
    fn test_cache_compression() {
        let mut cache = Cache::new();
        let key = 1i64;

        // 使用较大的数据测试压缩/解压缩
        let large_data: Vec<u8> = (0..1000).map(|i| (i % 256) as u8).collect();

        // 设置缓存
        assert!(cache.set(key, &large_data).is_ok());

        // 获取并验证数据完整性
        let retrieved = cache.get(key);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), large_data);
    }
}
