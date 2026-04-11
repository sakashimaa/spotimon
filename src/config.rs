use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    pub device: DeviceConfig,
    pub music_folder: PathBuf,
}

#[derive(Serialize, Deserialize)]
pub struct DeviceConfig {
    pub volume: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            device: DeviceConfig { volume: 50 },
            music_folder: dirs::home_dir().expect("No home dir").join("Music"),
        }
    }
}

pub fn write_default_config(path: &Path) -> AppConfig {
    let default_conf = AppConfig::default();
    let toml_str = toml::to_string(&default_conf).expect("Failed to serialize");
    fs::write(path, &toml_str).expect("Failed to write");

    default_conf
}
