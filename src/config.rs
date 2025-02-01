use anyhow::Context;
use std::{fs, path::PathBuf};

use directories_next::{self, BaseDirs};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    #[serde(default)]
    pub rofi: Rofi,
    #[serde(default)]
    pub cliphist: ClipHist,
    #[serde(default)]
    pub clipboard: Clipboard,
    #[serde(default = "default_image_mode_config")]
    pub image_mode_config: ModeConfig,
    #[serde(default = "default_text_mode_config")]
    pub text_mode_config: ModeConfig,
    #[serde(default = "default_delete_mode_config")]
    pub delete_mode_config: ModeConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Rofi {
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClipHist {
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Clipboard {
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModeConfig {
    pub title: String,
    pub shortcut: String,
    pub description: String,
}

pub fn load(path: &PathBuf) -> anyhow::Result<Config> {
    let config = fs::read_to_string(path).context("Error reading config file")?;
    toml::from_str(&config).context("Error parsing config file")
}

pub fn load_default() -> anyhow::Result<Config> {
    let dirs = BaseDirs::new().expect("Error getting base directories");

    let config_path = dirs.config_dir().join("rofi-cliphist.toml");

    load(&config_path)
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rofi: Rofi::default(),
            cliphist: ClipHist::default(),
            clipboard: Clipboard::default(),
            image_mode_config: ModeConfig {
                title: "Images".to_string(),
                shortcut: "Alt+i".to_string(),
                description: "Switch to images".to_string(),
            },
            text_mode_config: ModeConfig {
                title: "Texts".to_string(),
                shortcut: "Alt+d".to_string(),
                description: "Switch to text".to_string(),
            },
            delete_mode_config: ModeConfig {
                title: "Delete".to_string(),
                shortcut: "Alt+d".to_string(),
                description: "Delete entry".to_string(),
            },
        }
    }
}

impl Default for Rofi {
    fn default() -> Self {
        Self {
            path: "rofi".to_string(),
        }
    }
}

impl Default for ClipHist {
    fn default() -> Self {
        Self {
            path: "cliphist".to_string(),
        }
    }
}

impl Default for Clipboard {
    fn default() -> Self {
        Self {
            path: "wl-copy".to_string(),
        }
    }
}

fn default_image_mode_config() -> ModeConfig {
    ModeConfig {
        title: "Images".to_string(),
        shortcut: "Alt+i".to_string(),
        description: "Switch to images".to_string(),
    }
}

fn default_text_mode_config() -> ModeConfig {
    ModeConfig {
        title: "Texts".to_string(),
        shortcut: "Alt+d".to_string(),
        description: "Switch to text".to_string(),
    }
}

fn default_delete_mode_config() -> ModeConfig {
    ModeConfig {
        title: "Delete".to_string(),
        shortcut: "Alt+d".to_string(),
        description: "Delete entry".to_string(),
    }
}
