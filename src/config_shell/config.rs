use config::{Config as ConfigLoader, File};
use dirs::config_dir;
use log::error;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    ConfigSlint, HardwareConfigSlint, TaskbarConfigSlint, config_shell::components::taskbar::{TaskbarConfig, default_taskbar},
};
use crate::{
    InteractionConfigSlint, TrayConfigSlint, WindowConfigSlint,
    config_shell::components::{
        theme::{MaterialScheme, default_dark_scheme, default_light_scheme},
        tray::{TrayConfig, default_tray},
    },
};

#[derive(Serialize, Deserialize, Clone)]
pub struct InterractionConfig {
    pub animation_multiplier: f32,
    pub volume_scroll_step: u8,
    pub allow_overflow_volume: bool,
    pub brightness_scroll_step: u8,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WindowConfig {
    pub bar_popup_max_height: u16,
    pub bar_popup_screen_padding: u16,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HardwareConfig {
    pub brightness_device: String,
    pub hardware_specific_features: bool,
    pub sys_info_polling_duration: u16,
    pub temp_alert: u16,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub default_bar: String,
    pub icon_theme: String,
    pub default_display: String,
    pub fallback_display: String,
    pub window_config: WindowConfig,
    pub hardware_config: HardwareConfig,
    pub interaction_config: InterractionConfig,
    pub taskbar_config: TaskbarConfig,
    pub tray_config: TrayConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_bar: "default".to_string(),
            icon_theme: "Papirus-Dark".to_string(),
            default_display: "GIGA-BYTE TECHNOLOGY CO., LTD. G27QC A 0x00000439".to_string(),
            fallback_display: "eDP-1".to_string(),
            window_config: WindowConfig {
                bar_popup_max_height: 450,
                bar_popup_screen_padding: 4,
            },
            #[cfg(feature = "default_hardware")]
            hardware_config: HardwareConfig {
                brightness_device: "amdgpu_bl1".to_string(),
                hardware_specific_features: false,
                sys_info_polling_duration: 5,
                temp_alert: 85,
            },
            #[cfg(not(feature = "default_hardware"))]
            hardware_config: HardwareConfig {
                brightness_device: "amdgpu_bl1".to_string(),
                hardware_specific_features: true,
                sys_info_polling_duration: 1,
                temp_alert: 90,
            },
            interaction_config: InterractionConfig {
                animation_multiplier: 1.0,
                volume_scroll_step: 3,
                allow_overflow_volume: true,
                brightness_scroll_step: 5,
            },
            taskbar_config: default_taskbar(),
            tray_config: default_tray(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
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


fn parse_hex_color(hex: &str) -> Option<[u8; 3]> {
    let clean_hex = hex.trim_start_matches('#');
    let rgb_str = match clean_hex.len() {
        6 => clean_hex,
        8 => &clean_hex[2..],
        _ => return None,
    };

    let r = u8::from_str_radix(&rgb_str[0..2], 16).ok()?;
    let g = u8::from_str_radix(&rgb_str[2..4], 16).ok()?;
    let b = u8::from_str_radix(&rgb_str[4..6], 16).ok()?;

    Some([r, g, b])
}

fn json_color(json: &Value, key: &str) -> Option<[u8; 3]> {
    json.get(key)
        .and_then(|v| v.as_str())
        .and_then(parse_hex_color)
}

fn fetch_color(target: &Value, key: &str, alt_key: &str) -> Result<[u8; 3], Box<dyn std::error::Error>> {
    json_color(target, key)
        .or_else(|| json_color(target, alt_key))
        .ok_or_else(|| format!("Failed to parse required color key: {} / {}", key, alt_key).into())
}

fn parse_noctalia_palette(json: &Value, is_dark: bool) -> Result<MaterialScheme, Box<dyn std::error::Error>> {
    let target = if json.get("dark").is_some() || json.get("light").is_some() {
        if is_dark {
            json.get("dark").ok_or("Missing 'dark' section in palette JSON")?
        } else {
            json.get("light").ok_or("Missing 'light' section in palette JSON")?
        }
    } else {
        json
    };

    let primary = fetch_color(target, "mPrimary", "primary")?;
    let on_primary = fetch_color(target, "mOnPrimary", "on_primary")?;
    let secondary = fetch_color(target, "mSecondary", "secondary")?;
    let on_secondary = fetch_color(target, "mOnSecondary", "on_secondary")?;
    let tertiary = fetch_color(target, "mTertiary", "tertiary")?;
    let on_tertiary = fetch_color(target, "mOnTertiary", "on_tertiary")?;
    let error = fetch_color(target, "mError", "error")?;
    let on_error = fetch_color(target, "mOnError", "on_error")?;
    let surface = fetch_color(target, "mSurface", "surface")?;
    let on_surface = fetch_color(target, "mOnSurface", "on_surface")?;
    let surface_variant = fetch_color(target, "mSurfaceVariant", "surface_variant")?;
    let on_surface_variant = fetch_color(target, "mOnSurfaceVariant", "on_surface_variant")?;
    let outline = fetch_color(target, "mOutline", "outline")?;
    let shadow = fetch_color(target, "mShadow", "shadow")?;

    Ok(MaterialScheme {
        primary,
        surface_tint: primary,
        on_primary,
        primary_container: surface_variant,
        on_primary_container: on_surface_variant,
        secondary,
        on_secondary,
        secondary_container: surface_variant,
        on_secondary_container: on_surface_variant,
        tertiary,
        on_tertiary,
        tertiary_container: surface_variant,
        on_tertiary_container: on_surface_variant,
        error,
        on_error,
        error_container: surface_variant,
        on_error_container: on_surface_variant,
        background: surface,
        on_background: on_surface,
        surface,
        on_surface,
        surface_variant,
        on_surface_variant,
        outline,
        outline_variant: outline,
        shadow,
        scrim: shadow,
        inverse_surface: on_surface,
        inverse_on_surface: surface,
        inverse_primary: primary,
        primary_fixed: primary,
        on_primary_fixed: on_primary,
        primary_fixed_dim: primary,
        on_primary_fixed_variant: on_primary,
        secondary_fixed: secondary,
        on_secondary_fixed: on_secondary,
        secondary_fixed_dim: secondary,
        on_secondary_fixed_variant: on_secondary,
        tertiary_fixed: tertiary,
        on_tertiary_fixed: on_tertiary,
        tertiary_fixed_dim: tertiary,
        on_tertiary_fixed_variant: on_tertiary,
        surface_dim: surface,
        surface_bright: surface,
        surface_container_lowest: surface,
        surface_container_low: surface,
        surface_container: surface_variant,
        surface_container_high: surface_variant,
        surface_container_highest: surface_variant,
    })
}

fn fetch_noctalia_theme() -> Result<ThemeConfig, Box<dyn std::error::Error>> {
    let scheme_output = Command::new("noctalia")
        .args(["msg", "color-scheme-get"])
        .output()?;

    if !scheme_output.status.success() {
        return Err("`noctalia msg color-scheme-get` failed".into());
    }

    let scheme_str = String::from_utf8_lossy(&scheme_output.stdout);
    let mut parts = scheme_str.split_whitespace();
    let source = parts.next().ok_or("Invalid color scheme source")?;
    let palette_name = parts.collect::<Vec<&str>>().join(" ");

    if palette_name.is_empty() {
        return Err("Invalid color scheme name".into());
    }

    let base_dir = match source {
        "custom" => {
            let mut dir = config_dir().ok_or("Could not find config directory")?;
            dir.push("noctalia/palettes");
            dir
        }
        "community" => {
            let mut dir = dirs::home_dir().ok_or("Could not find home directory")?;
            dir.push(".local/state/noctalia/community-palettes");
            dir
        }
        "builtin" => PathBuf::from("/usr/share/noctalia/palettes"),
        other => return Err(format!("Unknown theme source: {}", other).into()),
    };

    let encoded_name = urlencoding::encode(&palette_name);
    let mut palette_path = base_dir.join(format!("{}.json", encoded_name));

    if !palette_path.exists() {
        palette_path = base_dir.join(format!("{}.json", palette_name));
    }

    let content = fs::read_to_string(&palette_path)
        .map_err(|e| format!("Failed to read palette at {:?}: {}", palette_path, e))?;

    let json: Value = serde_json::from_str(&content)?;

    let dark_scheme = parse_noctalia_palette(&json, true)?;
    let light_scheme = parse_noctalia_palette(&json, false)?;

    Ok(ThemeConfig {
        dark_scheme,
        light_scheme,
    })
}
pub fn load_or_create_theme_config() -> Result<ThemeConfig, Box<dyn std::error::Error>> {
    match fetch_noctalia_theme() {
        Ok(theme) => Ok(theme),
        Err(err) => {
            error!("Failed to fetch Noctalia theme ({}): using default theme", err);
            Ok(ThemeConfig::default())
        }
    }
}

#[derive(Clone)]
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

pub fn build_config_slint(config: &crate::config::AppConfig, window_bar_width: f32, bar_height: f32) -> ConfigSlint {
    ConfigSlint {
        hardware: HardwareConfigSlint {
            hardware_specific_features: config.config.hardware_config.hardware_specific_features,
            temp_alert: config.config.hardware_config.temp_alert as i32,
        },
        window: WindowConfigSlint {
            bar_height: bar_height as f32,
            bar_popup_max_height: config.config.window_config.bar_popup_max_height as f32,
            bar_popup_screen_padding: config.config.window_config.bar_popup_screen_padding as f32,
        },
        interaction: InteractionConfigSlint {
            animation_multiplier: config.config.interaction_config.animation_multiplier,
            volume_scroll_step: config.config.interaction_config.volume_scroll_step as i32,
            brightness_scroll_step: config.config.interaction_config.brightness_scroll_step as i32,
            sync_brightness: true,
        },
        taskbar: TaskbarConfigSlint {
            icon_size: config.config.taskbar_config.icon_size as f32,
            indicator_max_width: window_bar_width as f32
                * config.config.taskbar_config.indicator_max_width as f32,
            taskbar_max_width: window_bar_width as f32
                * config.config.taskbar_config.taskbar_max_width as f32,
        },
        tray: TrayConfigSlint {
            icon_size: config.config.tray_config.icon_size as f32,
            max_height: config.config.tray_config.max_menu_height as f32,
            width: config.config.tray_config.menu_width as f32,
            menu_icon_size: config.config.tray_config.menu_icon_size as f32,
        },
    }
}