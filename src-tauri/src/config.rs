use super::error::Error;
use super::Result;
use std::path::Path;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct AppConfig {
    pub(crate) database: DatabaseConfig,
}

impl AppConfig {
    /// 配置文件名
    const FILENAME: &'static str = "config.json";

    pub(crate) fn load() -> Result<AppConfig> {
        let path = Path::new(Self::FILENAME);
        if !path.exists() {
            std::fs::write(path, include_str!("../config.example.json"))?;
        }

        let config = std::fs::read_to_string(Self::FILENAME)?;
        serde_json::from_str(&config).map_err(Error::SerdeJson)
    }

    pub(crate) fn save(&self) -> Result<()> {
        let path = Path::new(Self::FILENAME);
        let config = serde_json::to_string_pretty(self)?;
        std::fs::write(path, config)?;
        Ok(())
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct DatabaseConfig {
    user: String,
    password: String,
    host: String,
    port: u16,
    database: String,
}

impl DatabaseConfig {
    pub(crate) fn url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            &self.user, &self.password, &self.host, &self.port, &self.database
        )
    }
}
