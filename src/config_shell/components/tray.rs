use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrayConfig {
    pub icon_size: u16,
    pub menu_icon_size: u16,
    pub menu_width: u16,
    pub max_menu_height: u16,
}

pub fn default_tray() -> TrayConfig {
    TrayConfig {
        icon_size: 16,
        menu_icon_size: 16,
        menu_width: 250,
        max_menu_height: 500,
    }
}
