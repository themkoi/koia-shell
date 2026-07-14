use std::sync::Arc;
use log::{error, info};
use wayle_media::MediaService;

use crate::barWindow;

pub async fn start_media_control(
    ui_weak: slint::Weak<barWindow>,
    media_service: Arc<MediaService>,
) {
    info!("starting media controller");

    slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            
            let media_play_pause = Arc::clone(&media_service);
            ui.on_media_play_pause(move || {
                let media = Arc::clone(&media_play_pause);
                tokio::spawn(async move {
                    if let Some(player) = media.active_player.get() {
                        if let Err(e) = player.play_pause().await {
                            error!("Failed to toggle play/pause: {:?}", e);
                        } else {
                            info!("Toggled play/pause for player: {}", player.identity.get());
                        }
                    }
                });
            });

            let media_next = Arc::clone(&media_service);
            ui.on_media_next(move || {
                let media = Arc::clone(&media_next);
                tokio::spawn(async move {
                    if let Some(player) = media.active_player.get() {
                        if let Err(e) = player.next().await {
                            error!("Failed to skip next track: {:?}", e);
                        } else {
                            info!("Skipped forward on player: {}", player.identity.get());
                        }
                    }
                });
            });

            let media_previous = Arc::clone(&media_service);
            ui.on_media_previous(move || {
                let media = Arc::clone(&media_previous);
                tokio::spawn(async move {
                    if let Some(player) = media.active_player.get() {
                        if let Err(e) = player.previous().await {
                            error!("Failed to skip to previous track: {:?}", e);
                        } else {
                            info!("Skipped backward on player: {}", player.identity.get());
                        }
                    }
                });
            });
        }
    })
    .unwrap();
}