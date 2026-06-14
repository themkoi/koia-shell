use std::path::Path;
use std::rc::Rc;
use std::{fs, thread};

use log::{debug, info};
use niri_ipc::{socket::Socket, Event, Request, Window};
use slint::{Image, ModelRc, VecModel};

use crate::services::taskbar::cache::{get_cache_folder, load_cache};
use crate::services::taskbar::serialize::SerializeState;
use crate::barWindow; // Import directly from here

pub fn run_taskbar(
    config: &crate::config::AppConfig,
    ui_weak: slint::Weak<barWindow>,
) -> thread::JoinHandle<()> {
    let config_internal = config.config.clone();
    let mut cache_folder = get_cache_folder();
    cache_folder.push("icons");
    fs::create_dir_all(&cache_folder).ok();

    thread::spawn(move || {
        let mut state = State::new();
        let mut icon_cache = load_cache();

        let icon_size = config_internal.taskbar_config.icon_size;
        let icon_theme = config_internal.icon_theme.clone();
        let separate_workspaces = config_internal.taskbar_config.seperate_workspaces;
        let sorting_mode = config_internal.taskbar_config.sorting_mode.clone();
        let check_cache_validity = true;

        let mut socket = match std::env::var("NIRI_SOCKET") {
            Ok(sock) => Socket::connect_to(sock).unwrap_or_else(|_| Socket::connect().unwrap()),
            Err(_) => Socket::connect().unwrap(),
        };

        let _ = socket.send(Request::EventStream);
        let mut events = socket.read_events();

        while let Ok(event) = events() {
            let loop_start = std::time::Instant::now();

            state.update_with_event(event, &config_internal);

            let serialized_state = SerializeState::from_parts(
                &state,
                &icon_size,
                &icon_theme,
                &separate_workspaces,
                &sorting_mode,
                &mut icon_cache,
                &check_cache_validity,
            );

            let mut paths_to_pass = Vec::new();

            for ws in &serialized_state.workspaces {
                let mut windows_paths = Vec::new();
                for w in &ws.windows {
                    let path_str = w.icon_path.to_string();
                    if !path_str.is_empty() {
                        let path = Path::new(&path_str);
                        if path.exists() {
                            let _ = Image::load_from_path(path);
                        }
                    }
                    windows_paths.push((
                        w.id,
                        w.app_id.clone(),
                        w.title.clone(),
                        path_str,
                        w.is_focused,
                    ));
                }
                paths_to_pass.push((ws.id, windows_paths));
            }

            let bg_elapsed = loop_start.elapsed();
            let ui_weak_clone = ui_weak.clone();

            slint::invoke_from_event_loop(move || {
                let dispatch_elapsed = loop_start.elapsed();
                let slint_closure_start = std::time::Instant::now();

                info!(
                    "PERF UI START -> Background Sync/Warmup: {:?} | Queue Latency: {:?}",
                    bg_elapsed,
                    dispatch_elapsed - bg_elapsed
                );

                if let Some(ui) = ui_weak_clone.upgrade() {
                    let workspaces_vec: Vec<crate::Workspace> = paths_to_pass
                        .into_iter()
                        .map(|(ws_id, windows)| {
                            let windows_vec: Vec<crate::Window> = windows
                                .into_iter()
                                .map(|(w_id, app_id, title, icon_path, is_focused)| {
                                    let icon_image = if !icon_path.is_empty() {
                                        Image::load_from_path(Path::new(&icon_path))
                                            .unwrap_or_default()
                                    } else {
                                        Image::default()
                                    };

                                    crate::Window {
                                        id: w_id,
                                        app_id,
                                        title,
                                        icon: icon_image,
                                        is_focused,
                                    }
                                })
                                .collect();

                            let windows_model = Rc::new(VecModel::from(windows_vec));

                            crate::Workspace {
                                id: ws_id,
                                windows: ModelRc::from(windows_model),
                            }
                        })
                        .collect();

                    let workspaces_model = Rc::new(VecModel::from(workspaces_vec));
                    ui.set_workspaces(ModelRc::from(workspaces_model));
                }

                info!(
                    "PERF UI END -> Total Main Event Execution: {:?}",
                    slint_closure_start.elapsed()
                );
            })
            .unwrap();
        }
    })
}

#[derive(Debug, Default, Clone)]
pub struct State {
    pub windows: Vec<Window>,
}

impl State {
    fn new() -> Self {
        Self::default()
    }

    fn update_with_event(&mut self, e: Event, config: &crate::config::Config) {
        match e {
            Event::WindowsChanged { windows } => {
                self.windows = windows;
            }

            Event::WindowOpenedOrChanged { window } => {
                if let Some(app_id) = window.app_id.as_ref() {
                    if config.taskbar_config.blacklist.contains(app_id) {
                        return;
                    }
                }

                if window.is_focused {
                    for w in self.windows.iter_mut() {
                        w.is_focused = false;
                    }
                }

                match self.windows.iter_mut().find(|w| w.id == window.id) {
                    Some(w) => *w = window,
                    None => self.windows.push(window),
                }
            }

            Event::WindowClosed { id } => {
                debug!("removing window {}", id);
                self.windows.retain(|w| w.id != id);
            }

            Event::WindowFocusChanged { id } => {
                for w in self.windows.iter_mut() {
                    w.is_focused = false;
                }

                if let Some(id) = id {
                    if let Some(w) = self.windows.iter_mut().find(|w| w.id == id) {
                        w.is_focused = true;
                    }
                }
            }

            _ => {}
        }
    }
}
