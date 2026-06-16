use wayle_audio::AudioService;

use crate::{barWindow, services::volume::{adjuster::start_volume_adjuster, listener::listen_volume_changes}};

mod listener;
mod adjuster;

pub async fn start_volume_management(ui_weak: slint::Weak<barWindow>) {
    let audio = AudioService::new().await.unwrap();
    listen_volume_changes(ui_weak.clone(),audio.clone()).await;
    start_volume_adjuster(ui_weak.clone(),audio).await;
}