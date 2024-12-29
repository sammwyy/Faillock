use anyhow::{Context, Result};
use serde_derive::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Service {
    pub log_file: String,
    pub log_trigger: String,
    pub log_ip_regex: String,
}

pub struct ServiceManager {
    pub services: HashMap<String, Service>,
}

impl ServiceManager {
    pub fn new() -> ServiceManager {
        ServiceManager {
            services: HashMap::new(),
        }
    }

    pub fn load_all(&mut self, workdir: &str) -> Result<usize> {
        let path = Path::new(workdir).join("services");

        let entries = fs::read_dir(&path)
            .with_context(|| format!("Failed to read services directory: {:?}", path))?;

        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file() {
                    let content = fs::read_to_string(entry.path()).with_context(|| {
                        format!("Failed to read service config file: {:?}", entry.path())
                    })?;

                    let config = toml::from_str::<Service>(&content).with_context(|| {
                        format!("Failed to parse service config: {:?}", entry.path())
                    })?;

                    // Name without extension
                    let key = entry
                        .file_name()
                        .to_str()
                        .context("Failed to get service name")?
                        .to_string()
                        .split('.')
                        .next()
                        .context("Failed to get service name")?
                        .to_string();

                    self.services.insert(key, config);
                }
            }
        }

        Ok(self.services.len())
    }

    pub fn get_service(&self, type_: &str) -> Option<&Service> {
        self.services.get(type_)
    }
}
