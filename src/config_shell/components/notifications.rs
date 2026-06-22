use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NotificationConfig {
    pub icon_size: u16,
    pub notification_width: u16,
    pub notification_max_height: u16,
    pub max_title_lenght: u16,
    pub max_text_lenght: u16,
    pub notification_timeout: u16,
    pub notification_never_timeout: u16,
    pub other_action_buttons: bool,
}

pub fn default_notificaiton() -> NotificationConfig {
    NotificationConfig {
        icon_size: 36,
        notification_width: 370,
        notification_max_height: 250,
        max_title_lenght: 100,
        max_text_lenght: 250,
        notification_timeout: 12,
        notification_never_timeout: 0,
        other_action_buttons: false,
    }
}
