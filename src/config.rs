use anyhow::{anyhow, Result};

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use std::path::PathBuf;

use hueclient::Bridge;
use std::ops::{Deref, DerefMut};
use std::{env, fs};

pub struct ValidatedBridge {
    bridge: Bridge,
}

impl ValidatedBridge {
    pub fn from_bridge(bridge: Bridge) -> Result<Self> {
        bridge.get_all_lights()?;

        Ok(Self { bridge })
    }
}

impl Deref for ValidatedBridge {
    type Target = Bridge;

    fn deref(&self) -> &Self::Target {
        &self.bridge
    }
}

impl DerefMut for ValidatedBridge {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bridge
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {
    device_type: String, // honestly this is entirely unnecessary
    bridge_ip: std::net::IpAddr,
    bridge_username: String,
}

impl BridgeConfig {
    pub fn generate_device_type() -> String {
        let mut rng = thread_rng();
        let chars: String = (0..5).map(|_| rng.sample(Alphanumeric) as char).collect();
        format!("twitchbrite#{}", chars)
    }

    /// create a bridge config from an _authorized_ Bridge
    pub fn from_bridge(device_type: String, bridge: &ValidatedBridge) -> Self {
        BridgeConfig {
            device_type,
            bridge_ip: bridge.ip,
            bridge_username: bridge.username.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitchConfig {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    bridge_config: Option<BridgeConfig>,
    twitch_config: Option<TwitchConfig>,
}

impl Config {
    fn get_config_path() -> PathBuf {
        let exe = env::current_exe().expect("Failed to find environment. Check permissions.");
        let dir = exe.parent().unwrap();
        let mut dir = dir.to_path_buf();
        dir.push("config.toml");
        dir
    }

    pub fn new() -> Self {
        Self {
            bridge_config: None,
            twitch_config: None,
        }
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path();
        let result = fs::read(&config_path)?;
        Ok(toml::from_slice(result.as_slice())?)
    }

    pub fn save(&self) {
        let config_path = Self::get_config_path();
        let bytes = toml::to_vec(&self).expect("Failed to serialize config.");
        fs::write(config_path, bytes).expect("Failed to save config to disk.");
    }
}
