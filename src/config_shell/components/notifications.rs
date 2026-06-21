use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NotificationConfig {
    pub icon_size: u16,
    pub notification_width: u16,
    pub notification_max_height: u16,
    pub notification_timeout: u16,
}

pub fn default_notificaiton() -> NotificationConfig {
    NotificationConfig {
        icon_size: 36,
        notification_width: 370,
        notification_max_height: 250,
        notification_timeout: 10,
    }
}
