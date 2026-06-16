use std::path::Path;
use std::rc::Rc;

use log::{debug, info};
use niri_ipc::{socket::Socket, Event, Request, Window};
use slint::{Image, Model, ModelRc, VecModel};

use crate::barWindow;
use crate::services::taskbar::cache::{get_cache_folder, load_cache};
use crate::services::taskbar::serialize::SerializeState;

pub async fn run_taskbar(
    config: &crate::config::AppConfig,
    ui_weak: slint::Weak<barWindow>,
) -> tokio::task::JoinHandle<()> {
    info!("starting taskbar");
    
    let mut cache_folder = get_cache_folder();
    cache_folder.push("icons");
    
    tokio::fs::create_dir_all(&cache_folder).await.unwrap();

    let config_internal = config.config.clone();
    let config_internal_task = config_internal.clone();
    let ui_weak_task = ui_weak.clone();

    tokio::spawn(async move {
        let mut state = State::new();
        let mut icon_cache = load_cache();

        let icon_size = config_internal_task.taskbar_config.icon_size;
        let icon_theme = config_internal_task.icon_theme.clone();
        let separate_workspaces = config_internal_task.taskbar_config.separate_workspaces;
        let sorting_mode = config_internal_task.taskbar_config.sorting_mode.clone();
        let check_cache_validity = true;

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Event>();

        tokio::task::spawn_blocking(move || {
            let mut socket = match std::env::var("NIRI_SOCKET") {
                Ok(sock) => Socket::connect_to(sock).unwrap_or_else(|_| Socket::connect().unwrap()),
                Err(_) => Socket::connect().unwrap(),
            };

            let _ = socket.send(Request::EventStream);
            let mut events = socket.read_events();

            while let Ok(event) = events() {
                if tx.send(event).is_err() {
                    break;
                }
            }
        });

        while let Some(event) = rx.recv().await {
            state.update_with_event(event, &config_internal_task);

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

            let ui_weak_clone = ui_weak_task.clone();

            slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_weak_clone.upgrade() {
                    let mut currently_hovered_id: Option<i32> = None;

                    for workspace in ui.get_workspaces().iter() {
                        for window in workspace.windows.iter() {
                            if window.is_hovered_slint {
                                currently_hovered_id = Some(window.id);
                                break;
                            }
                        }
                        if currently_hovered_id.is_some() {
                            break;
                        }
                    }

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

                                    let is_hovered = Some(w_id) == currently_hovered_id;

                                    crate::Window {
                                        id: w_id,
                                        app_id,
                                        title,
                                        icon: icon_image,
                                        is_focused,
                                        is_hovered_slint: is_hovered, 
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