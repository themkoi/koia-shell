use serde::{Serialize, Deserialize};
use std::{collections::HashMap, fs, path::PathBuf};
use dirs::cache_dir;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct CacheData {
    pub icon_path: String,
}

pub type CacheMap = HashMap<String, CacheData>;

pub fn get_cache_folder() -> PathBuf {
    let mut path = cache_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("koia-shell");
    path.push("taskbar-cache");
    path
}

pub fn get_cache_file() -> PathBuf {
    let path = get_cache_folder();
    let _ = fs::create_dir_all(&path);
    path.join("cache.toml")
}

pub fn load_cache() -> CacheMap {
    let path = get_cache_file();
    if let Ok(data) = fs::read_to_string(&path) {
        toml::from_str(&data).unwrap_or_default()
    } else {
        HashMap::new()
    }
}

pub fn save_cache(history: &CacheMap) {
    if let Ok(toml_str) = toml::to_string(history) {
        let _ = fs::write(get_cache_file(), toml_str);
    }
}

pub fn set_path(history: &mut CacheMap, appid: &str, icon_path: &str) -> bool {
    let entry = history.entry(appid.to_string()).or_default();
    if entry.icon_path == icon_path {
        return false;
    }
    entry.icon_path = icon_path.to_string();
    true
}