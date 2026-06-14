use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrayConfig {
    pub icon_size: u16,
    pub icon_menu_size: u16,
    pub menu_width: u16,
    pub menu_height: u16,
}

pub fn default_tray() -> TrayConfig {
    TrayConfig {
        icon_size: 16,
        icon_menu_size: 16,
        menu_width: 250,
        menu_height: 500,
    }
}
