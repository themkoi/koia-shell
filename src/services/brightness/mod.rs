use crate::{barWindow, services::brightness::{adjuster::start_brightness_adjuster, external::brightness::start_external_brightness, listener::listen_brightness_changes}};

mod adjuster;
mod listener;
mod external;

pub async fn start_brightness_management(
    config: &crate::config::AppConfig,
    ui_weak: slint::Weak<barWindow>,
) {
    listen_brightness_changes(&config, ui_weak.clone()).await;
    start_brightness_adjuster(&config, ui_weak.clone()).await;
    start_external_brightness(ui_weak).await;
}
