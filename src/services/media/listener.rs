use std::sync::Arc;
use futures::StreamExt;
use log::{info};
use slint::ToSharedString;
use wayle_media::MediaService;

use crate::barWindow;

fn update_media_ui(ui_weak: &slint::Weak<barWindow>, player: &Arc<wayle_media::core::player::Player>) {
    let ui_weak = ui_weak.clone();
    let title = player.metadata.title.get().to_string();
    let state = format!("{:?}", player.playback_state.get()); // e.g., "Playing", "Paused"

    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_mediaTitle(title.to_shared_string());
            ui.set_mediaPlaybackState(state.to_shared_string());
        }
    });
}

fn clear_media_ui(ui_weak: &slint::Weak<barWindow>) {
    let ui_weak = ui_weak.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_mediaTitle("".to_shared_string());
            ui.set_mediaPlaybackState("".to_shared_string());
        }
    });
}

pub async fn listen_media_changes(
    ui_weak: slint::Weak<barWindow>,
    media_service: Arc<MediaService>,
) {
    info!("starting media service listener");

    tokio::spawn(async move {
        let mut active_player_stream = media_service.active_player.watch();
        let mut current_watcher_task: Option<tokio::task::JoinHandle<()>> = None;

        if let Some(initial_player) = media_service.active_player.get() {
            update_media_ui(&ui_weak, &initial_player);
        } else {
            clear_media_ui(&ui_weak);
        }

        while let Some(active_player_opt) = active_player_stream.next().await {
            if let Some(task) = current_watcher_task.take() {
                task.abort();
            }

            match active_player_opt {
                Some(player) => {
                    let ui_weak_clone = ui_weak.clone();
                    let player_clone = Arc::clone(&player);

                    update_media_ui(&ui_weak_clone, &player_clone);

                    current_watcher_task = Some(tokio::spawn(async move {
                        let mut state_stream = player_clone.playback_state.watch();
                        let mut title_stream = player_clone.metadata.title.watch();

                        loop {
                            tokio::select! {
                                Some(_) = state_stream.next() => {
                                    update_media_ui(&ui_weak_clone, &player_clone);
                                }
                                Some(_) = title_stream.next() => {
                                    update_media_ui(&ui_weak_clone, &player_clone);
                                }
                            }
                        }
                    }));
                }
                None => {
                    clear_media_ui(&ui_weak);
                }
            }
        }
    });
}