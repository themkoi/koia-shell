use config::{Config as ConfigLoader, File};
use dirs::config_dir;
use log::error;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{InterractionConfigSlint, WindowConfigSlint, config_shell::components::theme::{
    MaterialScheme, default_dark_scheme, default_light_scheme
}};
use crate::{
    config_shell::components::taskbar::{default_taskbar, TaskbarConfig},
    ConfigSlint, TaskbarConfigSlint,
};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InterractionConfig {
    pub volume_scroll_step: u8,
    pub brightness_scroll_step: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WindowConfig {
    pub total_bar_height: u16,
    pub bar_height: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub icon_theme: String,
    pub default_display: String,
    pub window_config: WindowConfig,
    pub interraction_config: InterractionConfig,
    pub taskbar_config: TaskbarConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            icon_theme: "Papirus-Dark".to_string(),
            default_display: "DP-3".to_string(),
            window_config: WindowConfig { total_bar_height: 100, bar_height: 35 },
            interraction_config: InterractionConfig { volume_scroll_step: 3, brightness_scroll_step: 5 },
            taskbar_config: default_taskbar(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ThemeConfig {
    pub dark_scheme: MaterialScheme,
    pub light_scheme: MaterialScheme,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            dark_scheme: default_dark_scheme(),
            light_scheme: default_light_scheme(),
        }
    }
}

fn config_root() -> PathBuf {
    let mut path = config_dir().expect("Unable to locate config directory");
    path.push("koia-shell");
    fs::create_dir_all(&path).expect("Unable to create config directory");
    path
}

fn config_file() -> PathBuf {
    let mut path = config_root();
    path.push("config.toml");
    path
}

fn theme_file() -> PathBuf {
    let mut path = config_root();
    path.push("theme.toml");
    path
}

fn write_config<P: AsRef<Path>, T: Serialize>(path: P, config: &T) -> std::io::Result<()> {
    let toml_string = toml::to_string(config).expect("Failed to serialize config");

    fs::write(path, toml_string)
}

pub fn load_or_create_config() -> Result<Config, Box<dyn std::error::Error>> {
    let path = config_file();

    if !path.exists() {
        let default = Config::default();
        write_config(&path, &default)?;
        return Ok(default);
    }

    let loaded = ConfigLoader::builder()
        .add_source(File::from(path.as_path()))
        .build()
        .and_then(|c| c.try_deserialize::<Config>());

    match loaded {
        Ok(cfg) => Ok(cfg),
        Err(_) => {
            error!("failed loading config: continuing with default");
            let default = Config::default();
            Ok(default)
        }
    }
}

pub fn load_or_create_theme_config() -> Result<ThemeConfig, Box<dyn std::error::Error>> {
    let path = theme_file();

    if !path.exists() {
        let default = ThemeConfig::default();
        write_config(&path, &default)?;
        return Ok(default);
    }

    let loaded = ConfigLoader::builder()
        .add_source(File::from(path.as_path()))
        .build()
        .and_then(|c| c.try_deserialize::<ThemeConfig>());

    match loaded {
        Ok(cfg) => Ok(cfg),
        Err(_) => {
            error!("failed loading theme: continuing with default");
            let default = ThemeConfig::default();
            Ok(default)
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub config: Config,
    pub theme: ThemeConfig,
}

pub fn load_app_config() -> Result<AppConfig, Box<dyn std::error::Error>> {
    Ok(AppConfig {
        config: load_or_create_config()?,
        theme: load_or_create_theme_config()?,
    })
}

pub fn build_config_slint(
    config: &crate::config::AppConfig,
) -> ConfigSlint {
    ConfigSlint {
        window: WindowConfigSlint {
            bar_height: config.config.window_config.bar_height as f32,
        },
        interraction: InterractionConfigSlint {
            volume_scroll_step: config.config.interraction_config.volume_scroll_step as i32,
            brightness_scroll_step: config.config.interraction_config.brightness_scroll_step as i32,
        },
        taskbar: TaskbarConfigSlint {
            icon_size: config.config.taskbar_config.icon_size as f32,
            max_text_lenght: config.config.taskbar_config.max_text_lenght as f32,
        },
    }
}