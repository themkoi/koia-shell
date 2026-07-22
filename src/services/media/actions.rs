use log::{error, info};
use slint::{ComponentHandle, ToSharedString};
use std::sync::Arc;
use wayle_media::types::{LoopMode, PlaybackState, ShuffleMode};
use wayle_media::MediaService;

use crate::barWindow;

pub async fn start_media_control(
    ui_weak: slint::Weak<barWindow>,
    media_service: Arc<MediaService>,
) {
    info!("starting media controller");

    let service_for_select = Arc::clone(&media_service);

    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let select_ui_weak = ui.as_weak();
            let media_service_select = Arc::clone(&service_for_select);
            ui.on_select_player(move |selected_identity| {
                let media_service = Arc::clone(&media_service_select);
                let target_name = selected_identity.to_string();

                if let Some(ui) = select_ui_weak.upgrade() {
                    ui.set_selected_player(selected_identity.clone().to_shared_string());
                }

                tokio::spawn(async move {
                    if target_name == "Active Player" {
                        let players = media_service.player_list.get();
                        let active_target = players
                            .iter()
                            .find(|p| p.playback_state.get() == PlaybackState::Playing)
                            .or_else(|| players.first());

                        if let Some(target) = active_target {
                            let _ = media_service
                                .set_active_player(Some(target.id.clone()))
                                .await;
                            info!(
                                "Switched active player auto-track: {}",
                                target.identity.get()
                            );
                        } else {
                            let _ = media_service.set_active_player(None).await;
                            info!("No active players found.");
                        }
                    } else {
                        let players = media_service.player_list.get();
                        if let Some(target_player) =
                            players.iter().find(|p| p.identity.get() == target_name)
                        {
                            let _ = media_service
                                .set_active_player(Some(target_player.id.clone()))
                                .await;
                            info!("Switched active player to: {}", target_name);
                        }
                    }
                });
            });

            let media = Arc::clone(&media_service);
            ui.on_media_play_pause(move || {
                let media = Arc::clone(&media);
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

            let media = Arc::clone(&media_service);
            ui.on_media_next(move || {
                let media = Arc::clone(&media);
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

            let media = Arc::clone(&media_service);
            ui.on_media_previous(move || {
                let media = Arc::clone(&media);
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

            let media = Arc::clone(&media_service);
            ui.on_set_position(move |secs| {
                let media = Arc::clone(&media);
                tokio::spawn(async move {
                    if let Some(player) = media.active_player.get() {
                        let target_secs = secs.max(0) as i64;
                        let current_secs = player.position.get().as_secs() as i64;
                        let offset_secs = target_secs - current_secs;
                        let offset_micros = offset_secs * 1_000_000;

                        if let Err(e) = player.seek(offset_micros).await {
                            error!(
                                "Failed to seek to {}s on {}: {:?}",
                                secs,
                                player.identity.get(),
                                e
                            );
                        } else {
                            info!("Seeked to {}s on player: {}", secs, player.identity.get());
                        }
                    }
                });
            });

            let media = Arc::clone(&media_service);
            ui.on_toggle_shuffle(move |current_shuffle| {
                let media = Arc::clone(&media);
                tokio::spawn(async move {
                    if let Some(player) = media.active_player.get() {
                        let next_mode = match current_shuffle.as_str() {
                            "On" => ShuffleMode::Off,
                            "Off" => ShuffleMode::On,
                            _ => return,
                        };

                        if let Err(e) = player.set_shuffle_mode(next_mode).await {
                            error!("Failed to set shuffle mode to {:?}: {:?}", next_mode, e);
                        } else {
                            info!(
                                "Set shuffle mode to {:?} for player: {}",
                                next_mode,
                                player.identity.get()
                            );
                        }
                    }
                });
            });

            let media = Arc::clone(&media_service);
            ui.on_cycle_loop_status(move |current_status| {
                let media = Arc::clone(&media);
                tokio::spawn(async move {
                    if let Some(player) = media.active_player.get() {
                        let next_status = match current_status.as_str() {
                            "None" => LoopMode::Playlist,
                            "Playlist" => LoopMode::Track,
                            _ => LoopMode::None,
                        };

                        if let Err(e) = player.set_loop_mode(next_status).await {
                            error!("Failed to set loop mode to {:?}: {:?}", next_status, e);
                        } else {
                            info!(
                                "Updated loop mode to {:?} for player: {}",
                                next_status,
                                player.identity.get()
                            );
                        }
                    }
                });
            });
        }
    });
}