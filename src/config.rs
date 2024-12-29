use anyhow::{Context, Result};
use serde_derive::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::monitor::MonitorConfig;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub ban_command: String,
    pub monitors: Vec<MonitorConfig>,
}

impl Config {
    pub fn default() -> Self {
        Config {
            ban_command: String::from(""),
            monitors: Vec::new(),
        }
    }

    pub fn load(workdir: &str) -> Result<Self> {
        let path = Path::new(workdir).join("config.toml");

        if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read config: {:?}", path))?;
            let config = toml::from_str::<Config>(&content)
                .with_context(|| format!("Failed to parse config: {:?}", path))?;
            return Ok(config);
        }

        let default_config = Config::default();
        default_config.save(path.clone())?;
        Ok(default_config)
    }

    pub fn save(&self, path: PathBuf) -> Result<()> {
        let content = toml::to_string_pretty(self).context("Failed to serialize config to TOML")?;
        fs::write(path, content).context("Failed to write config to file")?;
        Ok(())
    }
}
