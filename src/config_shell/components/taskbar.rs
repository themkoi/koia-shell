use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub enum SortingMode {
    Default,
    AZ,
    Id,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TaskbarConfig {
    pub icon_size: u16,
    pub taskbar_max_width: f32,
    pub indicator_max_width: f32,
    pub separate_workspaces: bool,
    pub sorting_mode: SortingMode,
    pub check_cache_validity: bool,
    pub blacklist: Vec<String>,
}

pub fn default_taskbar() -> TaskbarConfig {
    TaskbarConfig {
        icon_size: 16,
        separate_workspaces: true,
        sorting_mode: SortingMode::AZ,
        check_cache_validity: false,
        blacklist: vec!["cosmic-wanderer"]
            .into_iter()
            .map(String::from)
            .collect(),
        taskbar_max_width: 0.5,
        indicator_max_width: 0.4,
    }
}
