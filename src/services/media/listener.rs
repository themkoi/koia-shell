use base64::Engine;
use futures::StreamExt;
use image::DynamicImage;
use log::info;
use slint::{Image, Rgba8Pixel, SharedPixelBuffer, ToSharedString, VecModel};
use std::collections::HashMap;
use std::io::Cursor;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use wayle_media::types::{LoopMode, PlaybackState, ShuffleMode};
use wayle_media::MediaService;

use crate::{barWindow, MediaInfo};

#[derive(Debug, Clone, PartialEq)]
enum CoverSource {
    Base64(String),
    Http(String),
    File(String),
}

#[derive(Clone)]
struct InternalPlayerState {
    title: String,
    artist: String,
    album: String,
    playback_state: PlaybackState,
    length_seconds: i32,
    position: Duration,
    can_go_next: bool,
    can_go_previous: bool,
    can_play: bool,
    shuffle_mode: ShuffleMode,
    loop_mode: LoopMode,
    cover_buffer: Option<SharedPixelBuffer<Rgba8Pixel>>,
    art_source: Option<CoverSource>,
}

impl Default for InternalPlayerState {
    fn default() -> Self {
        Self {
            title: String::new(),
            artist: String::new(),
            album: String::new(),
            playback_state: PlaybackState::Stopped,
            length_seconds: 0,
            position: Duration::ZERO,
            can_go_next: false,
            can_go_previous: false,
            can_play: false,
            shuffle_mode: ShuffleMode::Off,
            loop_mode: LoopMode::None,
            cover_buffer: None,
            art_source: None,
        }
    }
}

fn get_art_source(player: &Arc<wayle_media::core::player::Player>) -> Option<CoverSource> {
    let raw = player
        .metadata
        .cover_art
        .get()
        .filter(|s| !s.is_empty())
        .or_else(|| player.metadata.art_url.get().filter(|s| !s.is_empty()));

    let val = raw?;

    if val.starts_with("data:image/") {
        Some(CoverSource::Base64(val))
    } else if val.starts_with("http://") || val.starts_with("https://") {
        Some(CoverSource::Http(val))
    } else if let Some(stripped) = val.strip_prefix("file://") {
        Some(CoverSource::File(stripped.to_string()))
    } else {
        Some(CoverSource::File(val))
    }
}

async fn load_image_from_source_async(
    source: &CoverSource,
) -> Result<DynamicImage, Box<dyn std::error::Error + Send + Sync>> {
    match source {
        CoverSource::File(path) => {
            let path = path.clone();
            let img = tokio::task::spawn_blocking(move || {
                image::ImageReader::open(path)?.decode()
            })
            .await??;
            Ok(img)
        }
        CoverSource::Base64(data_uri) => {
            let data_uri = data_uri.clone();
            let img = tokio::task::spawn_blocking(move || {
                let base64_data = data_uri
                    .split_once(',')
                    .map(|(_, b64)| b64)
                    .unwrap_or(&data_uri);

                let decoded_bytes =
                    base64::engine::general_purpose::STANDARD.decode(base64_data)?;
                let img = image::ImageReader::new(Cursor::new(decoded_bytes))
                    .with_guessed_format()?
                    .decode()?;
                Ok::<DynamicImage, Box<dyn std::error::Error + Send + Sync>>(img)
            })
            .await??;
            Ok(img)
        }
        CoverSource::Http(url) => {
            let response = reqwest::get(url).await?;
            let bytes = response.bytes().await?;

            let img = tokio::task::spawn_blocking(move || {
                let img = image::ImageReader::new(Cursor::new(bytes))
                    .with_guessed_format()?
                    .decode()?;
                Ok::<DynamicImage, Box<dyn std::error::Error + Send + Sync>>(img)
            })
            .await??;

            Ok(img)
        }
    }
}

async fn process_cover_art(
    art_source: Option<CoverSource>,
) -> Option<SharedPixelBuffer<Rgba8Pixel>> {
    let source = art_source?;
    let loaded_image = load_image_from_source_async(&source).await.ok()?;

    tokio::task::spawn_blocking(move || {
        let rgba = loaded_image.to_rgba8();
        let (w, h) = rgba.dimensions();
        let mut buffer = SharedPixelBuffer::<Rgba8Pixel>::new(w, h);
        buffer.make_mut_bytes().copy_from_slice(rgba.as_raw());
        buffer
    })
    .await
    .ok()
}

fn set_media_position_ui(ui_weak: &slint::Weak<barWindow>, position: Duration) {
    let ui_weak = ui_weak.clone();
    let pos_secs = position.as_secs() as i32;

    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_media_position_seconds(pos_secs);
        }
    });
}

fn push_state_to_ui(ui_weak: &slint::Weak<barWindow>, state: &InternalPlayerState) {
    let media_data = MediaInfo {
        title: state.title.to_shared_string(),
        artist: state.artist.to_shared_string(),
        album: state.album.to_shared_string(),
        playback_state: format!("{:?}", state.playback_state).to_shared_string(),
        length_seconds: state.length_seconds,
        can_go_next: state.can_go_next,
        can_go_previous: state.can_go_previous,
        can_play: state.can_play,
        shuffle: format!("{:?}", state.shuffle_mode).to_shared_string(),
        loop_status: format!("{:?}", state.loop_mode).to_shared_string(),
    };

    let pos_secs = state.position.as_secs() as i32;
    let cover_buffer = state.cover_buffer.clone();
    let ui_weak = ui_weak.clone();

    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_mediaData(media_data);
            ui.set_media_position_seconds(pos_secs);
            if let Some(buffer) = cover_buffer {
                ui.set_mediaCover(Image::from_rgba8(buffer));
            } else {
                ui.set_mediaCover(Image::default());
            }
        }
    });
}

fn update_player_list_ui(
    ui_weak: &slint::Weak<barWindow>,
    media_service: &Arc<MediaService>,
) {
    let ui_weak = ui_weak.clone();

    let players = media_service.player_list.get();
    let mut player_names: Vec<slint::SharedString> = Vec::with_capacity(players.len() + 1);

    player_names.push("Active Player".into());
    for p in players.iter() {
        player_names.push(p.identity.get().to_shared_string());
    }

    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let model = Rc::new(VecModel::from(player_names));
            ui.set_player_list(model.into());
            ui.set_selected_player("Active Player".into());
        }
    });
}

fn clear_media_ui(ui_weak: &slint::Weak<barWindow>) {
    let ui_weak = ui_weak.clone();

    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_mediaData(MediaInfo::default());
            ui.set_mediaCover(Image::default());
            ui.set_media_position_seconds(0);
        }
    });
}

fn setup_ui_callbacks(ui: &barWindow, player: Arc<wayle_media::core::player::Player>) {
    let player_clone = player.clone();
    ui.on_set_position(move |secs| {
        let player = player_clone.clone();
        tokio::spawn(async move {
            let target = Duration::from_secs(secs.max(0) as u64);
            let _ = player.set_position(target).await;
        });
    });

    let player_clone = player.clone();
    ui.on_toggle_shuffle(move |current_shuffle| {
        let player = player_clone.clone();
        tokio::spawn(async move {
            let next_mode = match current_shuffle.as_str() {
                "On" => ShuffleMode::Off,
                "Off" => ShuffleMode::On,
                _ => return,
            };
            let _ = player.set_shuffle_mode(next_mode).await;
        });
    });

    let player_clone = player.clone();
    ui.on_cycle_loop_status(move |current_status| {
        let player = player_clone.clone();
        tokio::spawn(async move {
            let next_status = match current_status.as_str() {
                "None" => LoopMode::Playlist,
                "Playlist" => LoopMode::Track,
                _ => LoopMode::None,
            };
            let _ = player.set_loop_mode(next_status).await;
        });
    });

    let player_clone = player.clone();
    ui.on_media_play_pause(move || {
        let player = player_clone.clone();
        tokio::spawn(async move {
            let _ = player.play_pause().await;
        });
    });

    let player_clone = player.clone();
    ui.on_media_next(move || {
        let player = player_clone.clone();
        tokio::spawn(async move {
            let _ = player.next().await;
        });
    });

    let player_clone = player.clone();
    ui.on_media_previous(move || {
        let player = player_clone.clone();
        tokio::spawn(async move {
            let _ = player.previous().await;
        });
    });
}

pub async fn listen_media_changes(
    ui_weak: slint::Weak<barWindow>,
    media_service: Arc<MediaService>,
) {
    info!("starting media service listener");

    tokio::spawn(async move {
        let mut active_player_stream = media_service.active_player.watch();
        let mut players_stream = media_service.players_monitored();

        let player_states: Arc<Mutex<HashMap<String, InternalPlayerState>>> = Arc::new(Mutex::new(HashMap::new()));
        let mut player_tasks: HashMap<String, tokio::task::JoinHandle<()>> = HashMap::new();

        update_player_list_ui(&ui_weak, &media_service);

        let players = media_service.player_list.get();
        let initial_active = players
            .iter()
            .find(|p| p.playback_state.get() == PlaybackState::Playing)
            .or_else(|| players.first());

        if let Some(target) = initial_active {
            let _ = media_service.set_active_player(Some(target.id.clone())).await;
        }

        loop {
            tokio::select! {
                Some(_) = players_stream.next() => {
                    update_player_list_ui(&ui_weak, &media_service);

                    let current_players = media_service.player_list.get();
                    let active_identities: Vec<String> = current_players.iter().map(|p| p.identity.get()).collect();

                    {
                        let mut states = player_states.lock().unwrap();
                        states.retain(|identity, _| active_identities.contains(identity));
                    }

                    player_tasks.retain(|identity, handle| {
                        if !active_identities.contains(identity) {
                            handle.abort();
                            false
                        } else {
                            true
                        }
                    });

                    for player in current_players {
                        let identity = player.identity.get();
                        if !player_tasks.contains_key(&identity) {
                            let player_clone = Arc::clone(&player);
                            let media_service_ref = Arc::clone(&media_service);
                            let ui_weak_ref = ui_weak.clone();
                            let states_ref = Arc::clone(&player_states);

                            let identity_task = identity.clone();

                            let handle = tokio::spawn(async move {
                                let identity = identity_task;
                                let mut state_stream = player_clone.playback_state.watch();
                                let mut metadata_stream = player_clone.metadata.watch();
                                let mut position_stream = player_clone.position.watch();
                                let mut shuffle_stream = player_clone.shuffle_mode.watch();
                                let mut loop_stream = player_clone.loop_mode.watch();

                                let mut seeked_stream = match player_clone.seeked_signal().await {
                                    Ok(s) => Some(Box::pin(s)),
                                    Err(_) => None,
                                };

                                let mut timer = tokio::time::interval_at(
                                    tokio::time::Instant::now() + Duration::from_secs(1),
                                    Duration::from_secs(1),
                                );

                                {
                                    let mut states = states_ref.lock().unwrap();
                                    let state = states.entry(identity.clone()).or_default();
                                    state.title = player_clone.metadata.title.get();
                                    state.artist = player_clone.metadata.artist.get();
                                    state.album = player_clone.metadata.album.get();
                                    state.playback_state = player_clone.playback_state.get();
                                    state.position = player_clone.position.get();
                                    state.can_go_next = player_clone.can_go_next.get();
                                    state.can_go_previous = player_clone.can_go_previous.get();
                                    state.can_play = player_clone.can_play.get();
                                    state.shuffle_mode = player_clone.shuffle_mode.get();
                                    state.loop_mode = player_clone.loop_mode.get();
                                    if let Some(len) = player_clone.metadata.length.get() {
                                        state.length_seconds = len.as_secs() as i32;
                                    }
                                }

                                loop {
                                    tokio::select! {
                                        _ = timer.tick() => {
                                            let is_playing = player_clone.playback_state.get() == PlaybackState::Playing;
                                            if is_playing {
                                                let state_snapshot = {
                                                    let mut states = states_ref.lock().unwrap();
                                                    let state = states.entry(identity.clone()).or_default();
                                                    state.position += Duration::from_secs(1);
                                                    state.clone()
                                                };

                                                let is_active = media_service_ref.active_player.get()
                                                    .map(|p| p.identity.get() == identity)
                                                    .unwrap_or(false);

                                                if is_active {
                                                    set_media_position_ui(&ui_weak_ref, state_snapshot.position);
                                                }
                                            }
                                        }

                                        Some(new_pos) = async {
                                            if let Some(ref mut stream) = seeked_stream {
                                                stream.next().await
                                            } else {
                                                futures::future::pending().await
                                            }
                                        } => {
                                            let state_snapshot = {
                                                let mut states = states_ref.lock().unwrap();
                                                let state = states.entry(identity.clone()).or_default();
                                                state.position = new_pos;
                                                state.clone()
                                            };

                                            let is_active = media_service_ref.active_player.get()
                                                .map(|p| p.identity.get() == identity)
                                                .unwrap_or(false);

                                            if is_active {
                                                set_media_position_ui(&ui_weak_ref, state_snapshot.position);
                                            }
                                        }

                                        Some(new_pos) = position_stream.next() => {
                                            if new_pos > Duration::ZERO {
                                                let state_snapshot = {
                                                    let mut states = states_ref.lock().unwrap();
                                                    let state = states.entry(identity.clone()).or_default();
                                                    state.position = new_pos;
                                                    state.clone()
                                                };

                                                let is_active = media_service_ref.active_player.get()
                                                    .map(|p| p.identity.get() == identity)
                                                    .unwrap_or(false);

                                                if is_active {
                                                    set_media_position_ui(&ui_weak_ref, state_snapshot.position);
                                                }
                                            }
                                        }

                                        Some(playback_state) = state_stream.next() => {
                                            if playback_state == PlaybackState::Playing {
                                                let _ = media_service_ref.set_active_player(Some(player_clone.id.clone())).await;
                                            }

                                            let fetched_pos = player_clone.position().await.ok();

                                            let state_snapshot = {
                                                let mut states = states_ref.lock().unwrap();
                                                let state = states.entry(identity.clone()).or_default();
                                                state.playback_state = playback_state;
                                                if let Some(pos) = fetched_pos {
                                                    if pos > Duration::ZERO {
                                                        state.position = pos;
                                                    }
                                                }
                                                state.clone()
                                            };

                                            let is_active = media_service_ref.active_player.get()
                                                .map(|p| p.identity.get() == identity)
                                                .unwrap_or(false);

                                            if is_active {
                                                push_state_to_ui(&ui_weak_ref, &state_snapshot);
                                            }
                                        }

                                        Some(shuffle) = shuffle_stream.next() => {
                                            let state_snapshot = {
                                                let mut states = states_ref.lock().unwrap();
                                                let state = states.entry(identity.clone()).or_default();
                                                state.shuffle_mode = shuffle;
                                                state.clone()
                                            };

                                            let is_active = media_service_ref.active_player.get()
                                                .map(|p| p.identity.get() == identity)
                                                .unwrap_or(false);

                                            if is_active {
                                                push_state_to_ui(&ui_weak_ref, &state_snapshot);
                                            }
                                        }

                                        Some(loop_m) = loop_stream.next() => {
                                            let state_snapshot = {
                                                let mut states = states_ref.lock().unwrap();
                                                let state = states.entry(identity.clone()).or_default();
                                                state.loop_mode = loop_m;
                                                state.clone()
                                            };

                                            let is_active = media_service_ref.active_player.get()
                                                .map(|p| p.identity.get() == identity)
                                                .unwrap_or(false);

                                            if is_active {
                                                push_state_to_ui(&ui_weak_ref, &state_snapshot);
                                            }
                                        }

                                        Some(_) = metadata_stream.next() => {
                                            let new_art_source = get_art_source(&player_clone);
                                            let current_pos = player_clone.position().await.ok();
                                            let new_title = player_clone.metadata.title.get();

                                            let (needs_cover_update, art_source_to_load) = {
                                                let mut states = states_ref.lock().unwrap();
                                                let state = states.entry(identity.clone()).or_default();
                                                
                                                let title_changed = state.title != new_title;
                                                state.title = new_title;
                                                state.artist = player_clone.metadata.artist.get();
                                                state.album = player_clone.metadata.album.get();
                                                state.can_go_next = player_clone.can_go_next.get();
                                                state.can_go_previous = player_clone.can_go_previous.get();
                                                state.can_play = player_clone.can_play.get();

                                                if let Some(len) = player_clone.metadata.length.get() {
                                                    let secs = len.as_secs() as i32;
                                                    if secs > 0 {
                                                        state.length_seconds = secs;
                                                    }
                                                }

                                                if let Some(pos) = current_pos {
                                                    if pos > Duration::ZERO {
                                                        state.position = pos;
                                                    }
                                                }

                                                if title_changed {
                                                    if let Some(ref source) = new_art_source {
                                                        if state.art_source.as_ref() != Some(source) {
                                                            state.art_source = new_art_source.clone();
                                                            (true, new_art_source)
                                                        } else {
                                                            (false, None)
                                                        }
                                                    } else {
                                                        // Title changed and new track has no artwork: clear buffer to free memory
                                                        state.art_source = None;
                                                        state.cover_buffer = None;
                                                        (false, None)
                                                    }
                                                } else {
                                                    // Title did not change: only fetch if source updated to a valid cover
                                                    if new_art_source.is_some() && state.art_source != new_art_source {
                                                        state.art_source = new_art_source.clone();
                                                        (true, new_art_source)
                                                    } else {
                                                        (false, None)
                                                    }
                                                }
                                            };

                                            if needs_cover_update {
                                                let new_buffer = process_cover_art(art_source_to_load).await;
                                                let mut states = states_ref.lock().unwrap();
                                                let state = states.entry(identity.clone()).or_default();
                                                state.cover_buffer = new_buffer;
                                            }

                                            let state_snapshot = {
                                                let states = states_ref.lock().unwrap();
                                                states.get(&identity).cloned().unwrap_or_default()
                                            };

                                            let is_active = media_service_ref.active_player.get()
                                                .map(|p| p.identity.get() == identity)
                                                .unwrap_or(false);

                                            if is_active {
                                                push_state_to_ui(&ui_weak_ref, &state_snapshot);
                                            }
                                        }
                                    }
                                }
                            });

                            player_tasks.insert(identity, handle);
                        }
                    }
                }

                Some(active_player_opt) = active_player_stream.next() => {
                    match active_player_opt {
                        Some(player) => {
                            let identity = player.identity.get();
                            let player_clone = Arc::clone(&player);
                            let ui_weak_clone = ui_weak.clone();

                            let state_snapshot = {
                                let states = player_states.lock().unwrap();
                                states.get(&identity).cloned()
                            };

                            if let Some(state) = state_snapshot {
                                push_state_to_ui(&ui_weak_clone, &state);
                            } else {
                                let mut state = InternalPlayerState {
                                    title: player_clone.metadata.title.get(),
                                    artist: player_clone.metadata.artist.get(),
                                    album: player_clone.metadata.album.get(),
                                    playback_state: player_clone.playback_state.get(),
                                    position: player_clone.position.get(),
                                    can_go_next: player_clone.can_go_next.get(),
                                    can_go_previous: player_clone.can_go_previous.get(),
                                    can_play: player_clone.can_play.get(),
                                    shuffle_mode: player_clone.shuffle_mode.get(),
                                    loop_mode: player_clone.loop_mode.get(),
                                    ..Default::default()
                                };

                                if let Some(len) = player_clone.metadata.length.get() {
                                    state.length_seconds = len.as_secs() as i32;
                                }

                                let art_source = get_art_source(&player_clone);
                                state.art_source = art_source.clone();
                                state.cover_buffer = process_cover_art(art_source).await;

                                {
                                    let mut states = player_states.lock().unwrap();
                                    states.insert(identity, state.clone());
                                }

                                push_state_to_ui(&ui_weak_clone, &state);
                            }

                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(ui) = ui_weak_clone.upgrade() {
                                    setup_ui_callbacks(&ui, player_clone);
                                }
                            });
                        }
                        None => {
                            clear_media_ui(&ui_weak);
                        }
                    }
                }
            }
        }
    });
}