use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct NotificationConfig {
    pub icon_size: u16,
    pub notification_width: u16,
    pub notification_max_height: u16,
    pub max_title_lenght: u16,
    pub max_text_lenght: u16,
    pub notification_timeout: u16,
    pub notification_never_timeout: u16,
    pub other_action_buttons: bool,
    pub icon_overrides: HashMap<String, String>,
    pub html_formatting: Vec<String>,
}

pub fn default_notificaiton() -> NotificationConfig {
    let mut overrides = HashMap::new();

    overrides.insert("signal".to_string(), "signal-desktop".to_string());
    NotificationConfig {
        icon_size: 48,
        notification_width: 370,
        notification_max_height: 250,
        max_title_lenght: 100,
        max_text_lenght: 250,
        notification_timeout: 12,
        notification_never_timeout: 0,
        other_action_buttons: false,
        icon_overrides: overrides,
        html_formatting: vec!["KDE Connect"].into_iter().map(String::from).collect(),
    }
}
