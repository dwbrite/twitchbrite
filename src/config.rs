use anyhow::{anyhow, Result};

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use std::path::PathBuf;

use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub device_type: String, // honestly this is entirely unnecessary
    pub bridge_ip: std::net::IpAddr,
    pub bridge_username: String,
}

pub fn get_config_path() -> Result<PathBuf> {
    let exe = env::current_exe()?;
    let dir = exe.parent().ok_or(anyhow!("reeeeeee"))?;
    let mut dir = dir.to_path_buf();
    dir.push("config.toml");

    Ok(dir)
}

pub fn generate_device_type() -> String {
    let mut rng = thread_rng();
    let chars: String = (0..5).map(|_| rng.sample(Alphanumeric) as char).collect();
    format!("twitchbrite#{}", chars)
}
