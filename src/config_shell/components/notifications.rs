use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NotificationConfig {
    pub icon_size: u16,
    pub notification_width: u16,
    pub notification_height: u16,
}

pub fn default_notificaiton() -> NotificationConfig {
    NotificationConfig {
        icon_size: 16,
        notification_width: 300,
        notification_height: 150,
    }
}
