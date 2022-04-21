use anyhow::{anyhow, Result};
use hueclient::Bridge;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use std::{env, fs, thread};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub(crate) device_type: String, // honestly this is entirely unnecessary
    pub(crate) username: String,
}

fn get_config_path() -> Result<PathBuf> {
    let exe = env::current_exe()?;
    let dir = exe.parent().ok_or(anyhow!("reeeeeee"))?;
    let mut dir = dir.to_path_buf();
    dir.push("config.toml");

    Ok(dir)
}

fn generate_device_type() -> String {
    let mut rng = thread_rng();
    let chars: String = (0..5).map(|_| rng.sample(Alphanumeric) as char).collect();
    format!("twitchbrite#{}", chars)
}

fn register_bridge(device_type: &str) -> hueclient::Bridge {
    println!("press button on bridge to register this client");
    let mut bridge_result = hueclient::Bridge::discover_required().register_user(&device_type);
    while bridge_result.is_err() {
        bridge_result = hueclient::Bridge::discover_required().register_user(&device_type);
        thread::sleep(Duration::from_millis(500));
        println!("waiting ...")
    }

    bridge_result.unwrap()
}

fn generate_new_config(config_path: &PathBuf) -> Result<(Bridge, Config)> {
    let device_type = generate_device_type();
    let bridge = register_bridge(&device_type);

    let config = Config {
        device_type,
        username: bridge.username.clone(),
    };

    let mut file = fs::File::create(&config_path)?;

    file.write(toml::to_vec(&config)?.as_slice())?;
    return Ok((bridge, config));
}

impl Config {
    pub fn init() -> Result<(hueclient::Bridge, Config)> {
        // attempt to byte read config file
        let config_path = get_config_path()?;
        let result = fs::read(&config_path);

        if let Err(_) = &result {
            return generate_new_config(&config_path);
        }

        // deserialize existing config
        let config: Config = toml::from_slice(result.unwrap().as_slice())?; // TODO: better error, failed to parse config
        let bridge = hueclient::Bridge::discover_required().with_user(&config.username);
        Ok((bridge, config))
    }
}
