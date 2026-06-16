use crate::{barWindow, services::brightness::{adjuster::start_brightness_adjuster, listener::listen_brightness_changes}};

mod adjuster;
mod listener;

pub async fn start_brightness_management(
    config: &crate::config::AppConfig,
    ui_weak: slint::Weak<barWindow>,
) {
    listen_brightness_changes(&config, ui_weak.clone()).await;
    start_brightness_adjuster(&config, ui_weak).await;
}
