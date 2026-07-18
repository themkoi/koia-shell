use dirs::data_local_dir;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::{
fs,
path::{Path, PathBuf},
};


use crate::{SessionDataSlint, config_shell::config::Config};



#[derive(Serialize, Deserialize, Clone)]
pub struct SessionData {
    pub sync_brightness: bool,
    pub dark_mode: bool,
    pub caffeine: bool,
    pub dnd: bool,
}


impl Default for SessionData {
    fn default() -> Self {
        Self {
            sync_brightness: true,
            dark_mode: true,
            caffeine: false,
            dnd: false,  
        }
    }
}


fn data_root() -> PathBuf {
    let mut path = data_local_dir().expect("Unable to locate data directory");
    path.push("koia-shell");
    fs::create_dir_all(&path).expect("Unable to create data directory");
    path
}


fn session_data_file() -> PathBuf {
    let mut path = data_root();
    path.push("session.ron");
    path
}


fn write_session_data<P: AsRef<Path>>(
    path: P,
    data: &SessionData,
) -> Result<(), Box<dyn std::error::Error>> {
    let ron = ron::ser::to_string_pretty(data, ron::ser::PrettyConfig::default())?;
    fs::write(path, ron)?;
    Ok(())
}


pub fn load_or_create_session_data(
    config: &Config,
) -> Result<SessionData, Box<dyn std::error::Error>> {
    let path = session_data_file();


    if !path.exists() {
        let default = SessionData {
            sync_brightness: config.settings_config.sync_brightness,
            dark_mode: config.settings_config.dark_mode,
            caffeine: config.settings_config.caffeine,
            dnd: config.settings_config.dnd, 
        };


        write_session_data(&path, &default)?;
        return Ok(default);
    }


    let loaded = fs::read_to_string(&path)
        .ok()
        .and_then(|contents| ron::from_str::<SessionData>(&contents).ok());


    let mut data = match loaded {
        Some(data) => data,
        None => {
            error!("failed loading session data: continuing with default");


            SessionData {
                sync_brightness: config.settings_config.sync_brightness,
                dark_mode: config.settings_config.dark_mode,
                caffeine: config.settings_config.caffeine,
                dnd: config.settings_config.dnd,  
            }
        }
    };


    if !config.settings_config.persistent_sync_brightness {
        data.sync_brightness = config.settings_config.sync_brightness;
    }


    if !config.settings_config.persistent_dark_mode {
        data.dark_mode = config.settings_config.dark_mode;
    }


    if !config.settings_config.persistent_caffeine {
        data.caffeine = config.settings_config.caffeine;
    }


    if !config.settings_config.persistent_dnd {
        data.dnd = config.settings_config.dnd;
    }


    Ok(data)
}


pub fn save_session_data(
    config: &Config,
    mut data: SessionData,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Saving session data");
    
    if !config.settings_config.persistent_sync_brightness {
        data.sync_brightness = config.settings_config.sync_brightness;
    }


    if !config.settings_config.persistent_dark_mode {
        data.dark_mode = config.settings_config.dark_mode;
    }


    if !config.settings_config.persistent_caffeine {
        data.caffeine = config.settings_config.caffeine;
    }


    if !config.settings_config.persistent_dnd {
        data.dnd = config.settings_config.dnd;
    }


    write_session_data(session_data_file(), &data)
}


pub fn build_session_data_slint(data: &crate::data_shell::data::SessionData) -> SessionDataSlint {
    SessionDataSlint {
        sync_brightness: data.sync_brightness,
        dark_mode: data.dark_mode,
        caffeine: data.caffeine,
        dnd: data.dnd, 
    }
}


impl From<SessionDataSlint> for SessionData {
    fn from(value: SessionDataSlint) -> Self {
        Self {
            sync_brightness: value.sync_brightness,
            dark_mode: value.dark_mode,
            caffeine: value.caffeine,
            dnd: value.dnd, 
        }
    }
}
