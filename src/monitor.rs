use std::{
    collections::HashMap,
    path::Path,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use log::info;
use regex::Regex;
use serde_derive::{Deserialize, Serialize};

use crate::{file_reader::FileReader, file_watcher::FileWatcher, service::Service};

#[derive(Debug)]
pub struct Attempt {
    count: u32,
    last_attempt: Instant,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonitorConfig {
    #[serde(rename = "type")]
    pub type_: String,
    pub name: Option<String>,
    pub cleanup_interval: u64,
    pub max_attempts: u32,
    pub lock_time: Option<u64>,
    pub max_locks: Option<u32>,
}

pub struct Monitor {
    pub name: String,
    service: Service,
    monitor_config: MonitorConfig,
    ip_map: Arc<Mutex<HashMap<String, Attempt>>>,
    sender: mpsc::Sender<String>,
}

impl Monitor {
    pub fn new(
        service: Service,
        monitor_config: MonitorConfig,
        sender: mpsc::Sender<String>,
    ) -> Monitor {
        let name = monitor_config
            .name
            .clone()
            .unwrap_or(monitor_config.type_.clone());
        let ip_map = Arc::new(Mutex::new(HashMap::new()));

        Monitor {
            service,
            monitor_config,
            ip_map,
            name,
            sender,
        }
    }

    pub fn run(&self) -> Result<()> {
        let log_path = self.service.log_file.clone();
        let ban_regex =
            Regex::new(&self.service.log_trigger).with_context(|| "Invalid ban message regex")?;
        let ip_regex =
            Regex::new(&self.service.log_ip_regex).with_context(|| "Invalid IP regex")?;

        let file_path = Path::new(&self.service.log_file).to_path_buf();
        let mut watcher: FileWatcher = FileWatcher::new(&file_path);
        watcher.init().with_context(|| "Failed to create watcher")?;

        let max_attempts = self.monitor_config.max_attempts;
        let cleanup_interval = self.monitor_config.cleanup_interval;
        info!("Starting monitor: {}", self.name);

        let monitor_clone = self.clone();

        thread::spawn(move || loop {
            match watcher.wait_for_newlines() {
                Ok(result) => {
                    if let Some(new_lines) = result {
                        for line in new_lines {
                            if ban_regex.is_match(&line) {
                                if let Some(ip) =
                                    ip_regex.find(&line).map(|m| m.as_str().to_string())
                                {
                                    let mut ip_map = monitor_clone.ip_map.lock().unwrap();
                                    let entry = ip_map.entry(ip.clone()).or_insert(Attempt {
                                        count: 0,
                                        last_attempt: Instant::now(),
                                    });
                                    entry.count += 1;
                                    entry.last_attempt = Instant::now();

                                    info!(
                                        "Attempt {} for IP: {} on service: {}",
                                        entry.count, ip, monitor_clone.name
                                    );

                                    if entry.count >= max_attempts {
                                        info!(
                                            "IP reached max attempts {} from service: {}",
                                            ip, monitor_clone.name
                                        );
                                        monitor_clone
                                            .sender
                                            .send(ip.clone())
                                            .context("Failed to send IP");
                                        ip_map.remove(&ip);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(_) => (),
            }
        });

        // Cleanup loop
        let ip_map_clone = self.ip_map.clone();
        thread::spawn(move || loop {
            thread::sleep(std::time::Duration::from_secs(cleanup_interval));
            let mut ip_map = ip_map_clone.lock().unwrap();
            ip_map.retain(|_, attempt| attempt.last_attempt.elapsed().as_secs() < cleanup_interval);
        });

        Ok(())
    }
}

impl Clone for Monitor {
    fn clone(&self) -> Self {
        Monitor {
            service: self.service.clone(),
            monitor_config: self.monitor_config.clone(),
            ip_map: Arc::clone(&self.ip_map),
            name: self.name.clone(),
            sender: self.sender.clone(),
        }
    }
}

pub struct MonitorManager {
    monitors: HashMap<String, Monitor>,
}

impl MonitorManager {
    pub fn new() -> MonitorManager {
        MonitorManager {
            monitors: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.monitors.len()
    }

    pub fn add_monitor(&mut self, monitor: Monitor) {
        self.monitors.insert(monitor.name.clone(), monitor);
    }

    pub fn run_all(&self) -> Result<()> {
        for monitor in self.monitors.values() {
            monitor.run()?;
        }

        Ok(())
    }
}
