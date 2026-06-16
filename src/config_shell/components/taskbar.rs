use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SortingMode {
    Default,
    AZ,
    Id,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskbarConfig {
    pub icon_size: u16,
    pub max_text_lenght: u16,
    pub separate_workspaces: bool,
    pub sorting_mode: SortingMode,
    pub check_cache_validity: bool,
    pub blacklist: Vec<String>,
}

pub fn default_taskbar() -> TaskbarConfig {
    TaskbarConfig {
        icon_size: 16,
        max_text_lenght: 100,
        separate_workspaces: true,
        sorting_mode: SortingMode::AZ,
        check_cache_validity: false,
        blacklist: vec!["cosmic-wanderer"]
            .into_iter()
            .map(String::from)
            .collect(),
    }
}
