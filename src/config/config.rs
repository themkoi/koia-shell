use config::{Config as ConfigLoader, File};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use slint::Color;
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::config::theme_defaults::{
    default_dark_scheme,
    default_light_scheme,
    MaterialScheme,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub use_dark_theme: bool,
    pub dark_scheme: MaterialScheme,
    pub light_scheme: MaterialScheme,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            use_dark_theme: true,
            dark_scheme: default_dark_scheme(),
            light_scheme: default_light_scheme(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

pub fn config_color_to_slint(c: &ConfigColor) -> Color {
    Color::from_argb_u8(c.alpha, c.red, c.green, c.blue)
}

fn get_config_file() -> PathBuf {
    let mut path = config_dir().expect("Unable to locate config directory");

    path.push("cosmic-wanderer");
    fs::create_dir_all(&path).expect("Unable to create config directory");

    path.push("config.toml");
    path
}

fn write_config<P: AsRef<Path>>(path: P, config: &Config) -> std::io::Result<()> {
    let toml_string =
        toml::to_string_pretty(config).expect("Failed to serialize config");

    fs::write(path, toml_string)
}

pub fn load_or_create_config() -> Result<Config, Box<dyn std::error::Error>> {
    let path = get_config_file();

    if !path.exists() {
        let default = Config::default();

        write_config(&path, &default)?;

        return Ok(default);
    }

    let loaded = ConfigLoader::builder()
        .add_source(File::from(path.as_path()))
        .build()?
        .try_deserialize::<Config>()?;

    Ok(loaded)
}