use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub app: AppSection,
    pub ui: UiSection,
    pub storage: StorageSection,
    pub ordering: OrderingSection,
    pub logging: LoggingSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSection {
    pub name: String,
    pub default_board_name: String,
    pub seed_default_columns: bool,
    pub default_columns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiSection {
    pub start_route: String,
    pub enable_animations: bool,
    pub dense_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StorageSection {
    pub sqlite_filename: String,
    pub journal_mode: String,
    pub synchronous: String,
    pub foreign_keys: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OrderingSection {
    pub step: i64,
    pub min_gap: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingSection {
    pub level: String,
    pub json: bool,
    pub write_to_file: bool,
    pub log_dirname: String,
    pub max_log_files: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app: AppSection::default(),
            ui: UiSection::default(),
            storage: StorageSection::default(),
            ordering: OrderingSection::default(),
            logging: LoggingSection::default(),
        }
    }
}

impl Default for AppSection {
    fn default() -> Self {
        Self {
            name: "Oxban".to_string(),
            default_board_name: "My Board".to_string(),
            seed_default_columns: true,
            default_columns: vec!["To do".to_string(), "Doing".to_string(), "Done".to_string()],
        }
    }
}

impl Default for UiSection {
    fn default() -> Self {
        Self {
            start_route: "/".to_string(),
            enable_animations: true,
            dense_mode: false,
        }
    }
}

impl Default for StorageSection {
    fn default() -> Self {
        Self {
            sqlite_filename: "oxban.sqlite3".to_string(),
            journal_mode: "WAL".to_string(),
            synchronous: "NORMAL".to_string(),
            foreign_keys: true,
        }
    }
}

impl Default for OrderingSection {
    fn default() -> Self {
        Self {
            step: 1_000_000,
            min_gap: 4,
        }
    }
}

impl Default for LoggingSection {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            json: false,
            write_to_file: true,
            log_dirname: "logs".to_string(),
            max_log_files: 10,
        }
    }
}

#[tracing::instrument(skip(default_text), fields(config_dir = %config_dir.display(), filename = filename))]
pub fn load_or_init_config(
    config_dir: &Path,
    filename: &str,
    default_text: &str,
) -> anyhow::Result<AppConfig> {
    std::fs::create_dir_all(config_dir)?;
    let path = config_dir.join(filename);

    if !path.exists() {
        tracing::warn!(path = %path.display(), "configuration missing; writing default file");
        std::fs::write(&path, default_text)?;
    }

    let raw = std::fs::read_to_string(&path)?;
    match toml::from_str::<AppConfig>(&raw) {
        Ok(cfg) => Ok(cfg),
        Err(error) => {
            tracing::error!(%error, path = %path.display(), "failed to parse config; falling back to defaults");
            Ok(AppConfig::default())
        }
    }
}

pub fn default_config_text() -> &'static str {
    include_str!("../../oxban.toml")
}

pub fn log_file_path(config_dir: &Path, log_dirname: &str) -> PathBuf {
    config_dir.join(log_dirname).join("oxban.log")
}
