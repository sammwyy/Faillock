use anyhow::{bail, Context, Result};
use config::Config;
use env_logger::{self, Env};
use log::{error, info};
use monitor::{Monitor, MonitorManager};
use service::ServiceManager;
use std::{sync::mpsc, thread, time::Duration};

mod config;
mod file_reader;
mod file_watcher;
mod monitor;
mod service;

pub fn main() -> Result<()> {
    let env = Env::default().filter_or("FILTER", "info");
    env_logger::init_from_env(env);

    let workdir = "./";

    let (tx, rx) = mpsc::channel::<String>();
    let mut services = ServiceManager::new();
    let mut monitors = MonitorManager::new();

    let loaded = services.load_all(workdir)?;
    info!("Loaded {} services.", loaded);

    let config = Config::load(workdir).context("Failed to load configuration")?;
    for monitor_config in config.monitors.iter() {
        if let Some(service) = services.get_service(&monitor_config.type_) {
            let monitor = Monitor::new(service.clone(), monitor_config.clone(), tx.clone());
            monitors.add_monitor(monitor);
        } else {
            bail!("Service {} not found", monitor_config.type_);
        }
    }

    info!("Running {} monitors.", monitors.len());
    monitors.run_all()?;

    loop {
        match rx.recv() {
            Ok(ip) => {
                let command = config.ban_command.replace("{ip}", &ip);
                let output = std::process::Command::new("bash")
                    .arg("-c")
                    .arg(&command)
                    .output()
                    .expect("failed to execute process");

                if !output.status.success() {
                    error!(
                        "Command failed: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
            }
            Err(_) => (),
        }
    }
}
