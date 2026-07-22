use std::time::Duration;

use wayle_media::MediaServiceBuilder;

use crate::{
    barWindow,
    services::media::{actions::start_media_control, listener::listen_media_changes},
};

mod actions;
mod listener;

pub async fn start_media_management(
    ui_weak: slint::Weak<barWindow>,
) {
    tokio::spawn(async move {
        let media_service = MediaServiceBuilder::new().position_poll_interval(Duration::from_secs(1))
            .build()
            .await
            .expect("Failed to initialize MediaService");
        listen_media_changes(ui_weak.clone(), media_service.clone()).await;

        start_media_control(ui_weak, media_service).await;
    });
}
