use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

    // Window app_id -> desktop icon/app id override
    pub icon_aliases: HashMap<String, String>,
}

pub fn default_taskbar() -> TaskbarConfig {
    let mut icon_aliases = HashMap::new();

    icon_aliases.insert(
        "signal".into(),
        "signal-desktop".into(),
    );

    icon_aliases.insert(
        "* - OrcaSlicer".into(),
        "orcaslicer".into(),
    );

    TaskbarConfig {
        icon_size: 16,
        separate_workspaces: true,
        sorting_mode: SortingMode::AZ,
        check_cache_validity: false,

        blacklist: vec!["cosmic-wanderer"]
            .into_iter()
            .map(String::from)
            .collect(),

        icon_aliases,

        taskbar_max_width: 0.5,
        indicator_max_width: 0.4,
    }
}