use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub database_path: PathBuf,
    pub history_paths: Vec<PathBuf>,
    pub redaction_enabled: bool,
    pub auto_import: bool,
    pub danger_threshold: f32,
    pub experiment_detection: bool,
}

impl Default for Config {
    fn default() -> Self {
        let _config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("whiskerlog");

        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("whiskerlog");

        Self {
            database_path: data_dir.join("history.db"),
            history_paths: vec![
                dirs::home_dir().unwrap_or_default().join(".bash_history"),
                dirs::home_dir().unwrap_or_default().join(".zsh_history"),
                dirs::home_dir()
                    .unwrap_or_default()
                    .join(".local/share/fish/fish_history"),
            ],
            redaction_enabled: true,
            auto_import: true,
            danger_threshold: 0.7,
            experiment_detection: true,
        }
    }
}

impl Config {
    pub fn load_or_create() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("whiskerlog");

        let config_path = config_dir.join("config.toml");

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("whiskerlog");

        std::fs::create_dir_all(&config_dir)?;

        let config_path = config_dir.join("config.toml");
        let content = toml::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;

        // Also ensure data directory exists
        if let Some(parent) = self.database_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        Ok(())
    }
}
