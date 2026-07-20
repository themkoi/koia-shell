use config::{Config as ConfigLoader, File};
use dirs::config_dir;
use log::error;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    CommandsConfigSlint, ConfigSlint, HardwareConfigSlint, TaskbarConfigSlint,
    config_shell::components::taskbar::{TaskbarConfig, default_taskbar},
};
use crate::{
    InteractionConfigSlint, NoticificationConfigSlint, TrayConfigSlint, WindowConfigSlint,
    config_shell::components::{
        notifications::{NotificationConfig, default_notificaiton},
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
    pub bar_height: u16,
    pub bar_popup_max_height: u16,
    pub bar_popup_screen_padding: u16,
    pub notification_screen: String,
    pub notification_window_width: u16,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HardwareConfig {
    pub brightness_device: String,
    pub hardware_specific_features: bool,
    pub sys_info_polling_duration: u16,
    pub temp_alert: u16,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SettingsConfig {
    pub persistent_sync_brightness: bool,
    pub sync_brightness: bool,
    pub persistent_dark_mode: bool,
    pub dark_mode: bool,
    pub persistent_caffeine: bool,
    pub caffeine: bool,
    pub persistent_dnd: bool,
    pub dnd: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CommandConfig {
    pub shutdown: String,
    pub reboot: String,
    pub lock: String,
    pub suspend: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub icon_theme: String,
    pub default_display: String,
    pub fallback_display: String,
    pub profile_icon: String,
    pub commands_config: CommandConfig,
    pub window_config: WindowConfig,
    pub hardware_config: HardwareConfig,
    pub interaction_config: InterractionConfig,
    pub settings_config: SettingsConfig,
    pub taskbar_config: TaskbarConfig,
    pub tray_config: TrayConfig,
    pub notification_config: NotificationConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            icon_theme: "Papirus-Dark".to_string(),
            default_display: "GIGA-BYTE TECHNOLOGY CO., LTD. G27QC A 0x00000439".to_string(),
            fallback_display: "eDP-1".to_string(),
            profile_icon: "$HOME/Pictures/pfp/pfp.png".to_string(),
            commands_config: CommandConfig {
                shutdown: "systemctl poweroff".to_string(),
                reboot: "systemctl reboot".to_string(),
                lock: "$HOME/Documents/scripts/niri/lock.sh".to_string(),
                suspend: "systemctl suspend".to_string(),
            },
            window_config: WindowConfig {
                bar_height: 38,
                bar_popup_max_height: 450,
                bar_popup_screen_padding: 4,
                notification_screen: "eDP-1".to_string(),
                notification_window_width: 400,
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
            settings_config: SettingsConfig {
                persistent_sync_brightness: true,
                sync_brightness: true,
                persistent_dark_mode: true,
                dark_mode: true,
                persistent_caffeine: false,
                caffeine: false,
                persistent_dnd: false,
                dnd: false,
            },
            taskbar_config: default_taskbar(),
            tray_config: default_tray(),
            notification_config: default_notificaiton(),
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

pub fn build_config_slint(config: &crate::config::AppConfig, window_bar_width: f32) -> ConfigSlint {
    ConfigSlint {
        commands: CommandsConfigSlint {
            shutdown: config.config.commands_config.shutdown.clone().into(),
            reboot: config.config.commands_config.reboot.clone().into(),
            lock: config.config.commands_config.lock.clone().into(),
            suspend: config.config.commands_config.suspend.clone().into(),
        },
        hardware: HardwareConfigSlint {
            hardware_specific_features: config.config.hardware_config.hardware_specific_features,
            temp_alert: config.config.hardware_config.temp_alert as i32,
        },
        window: WindowConfigSlint {
            bar_height: config.config.window_config.bar_height as f32,
            bar_popup_max_height: config.config.window_config.bar_popup_max_height as f32,
            bar_popup_screen_padding: config.config.window_config.bar_popup_screen_padding as f32,
        },
        interaction: InteractionConfigSlint {
            animation_multiplier: config.config.interaction_config.animation_multiplier,
            volume_scroll_step: config.config.interaction_config.volume_scroll_step as i32,
            brightness_scroll_step: config.config.interaction_config.brightness_scroll_step as i32,
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
        notification: NoticificationConfigSlint {
            icon_size: config.config.notification_config.icon_size as f32,
            notification_max_height: config.config.notification_config.notification_max_height
                as f32,
            notification_width: config.config.notification_config.notification_width as f32,
            other_action_buttons: config.config.notification_config.other_action_buttons,
        },
    }
}
