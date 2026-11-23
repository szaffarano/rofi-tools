use anyhow::Context;
use directories_next::BaseDirs;
use log::debug;
use roto_core::config::{ModeConfig, Rofi};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

/// CLI configuration for rofi-clipcat
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    #[serde(default)]
    pub rofi: Rofi,
    #[serde(default)]
    pub clipcat: Clipcat,
    #[serde(default = "roto_core::config::default_image_mode_config")]
    pub image_mode_config: ModeConfig,
    #[serde(default = "roto_core::config::default_text_mode_config")]
    pub text_mode_config: ModeConfig,
    #[serde(default = "roto_core::config::default_delete_mode_config")]
    pub delete_mode_config: ModeConfig,
    #[serde(default = "roto_core::config::default_delete_previous_config")]
    pub delete_previous_config: ModeConfig,
    #[serde(default = "roto_core::config::default_delete_next_config")]
    pub delete_next_config: ModeConfig,
    #[serde(default = "roto_core::config::default_toggle_mode_config")]
    pub toggle_mode_config: ModeConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Clipcat {
    #[serde(default = "default_endpoint_string")]
    pub endpoint: String,
    #[serde(default)]
    pub access_token: Option<String>,
}

/// Load configuration from a file
pub fn load(path: &PathBuf) -> anyhow::Result<Config> {
    debug!("Loading config from file: {path:?}");
    let config = fs::read_to_string(path).context("Error reading config file")?;
    toml::from_str(&config).context("Error parsing config file")
}

/// Load the default configuration
pub fn load_default() -> anyhow::Result<Config> {
    debug!("Loading config from the default location");
    let dirs = BaseDirs::new().expect("Error getting base directories");
    let config_path = dirs.config_dir().join("rofi-clipcat.toml");
    load(&config_path)
}

impl Default for Config {
    fn default() -> Self {
        debug!("Creating default config");
        Self {
            rofi: Rofi::default(),
            clipcat: Clipcat::default(),
            image_mode_config: roto_core::config::default_image_mode_config(),
            text_mode_config: roto_core::config::default_text_mode_config(),
            delete_mode_config: roto_core::config::default_delete_mode_config(),
            delete_previous_config: roto_core::config::default_delete_previous_config(),
            delete_next_config: roto_core::config::default_delete_next_config(),
            toggle_mode_config: roto_core::config::default_toggle_mode_config(),
        }
    }
}

impl Default for Clipcat {
    fn default() -> Self {
        Self {
            endpoint: default_endpoint_string(),
            access_token: None,
        }
    }
}

fn default_endpoint_string() -> String {
    clipcat_base::config::default_server_endpoint().to_string()
}
