use std::path::Path;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use log::{debug, info};
use niri_ipc::{socket::Socket, Event, Request, Window};
use slint::{Image, Model, ModelRc, VecModel};

use crate::barWindow;
use crate::services::taskbar::cache::{get_cache_folder, load_cache};
use crate::services::taskbar::serialize::SerializeState;

thread_local! {
    static WORKSPACES_MODEL: RefCell<Rc<VecModel<crate::Workspace>>> = RefCell::new(Rc::new(VecModel::default()));
    static WINDOW_MODELS: RefCell<HashMap<i32, Rc<VecModel<crate::Window>>>> = RefCell::new(HashMap::new());
}

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
                    WORKSPACES_MODEL.with(|ws_cell| {
                        WINDOW_MODELS.with(|win_cell| {
                            let ws_model = ws_cell.borrow().clone();
                            let mut win_map = win_cell.borrow_mut();

                            if ws_model.row_count() == 0 && !paths_to_pass.is_empty() {
                                ui.set_workspaces(ModelRc::from(ws_model.clone()));
                            }

                            for (ws_id, windows) in &paths_to_pass {
                                let win_model = win_map.entry(*ws_id).or_insert_with(|| Rc::new(VecModel::default()));
                                
                                while win_model.row_count() > windows.len() {
                                    win_model.remove(win_model.row_count() - 1);
                                }
                                
                                for (idx, (w_id, app_id, title, icon_path, is_focused)) in windows.iter().enumerate() {
                                    let icon_image = if !icon_path.is_empty() {
                                        Image::load_from_path(Path::new(icon_path)).unwrap_or_default()
                                    } else {
                                        Image::default()
                                    };

                                    let win_data = crate::Window {
                                        id: *w_id as i32,
                                        app_id: app_id.clone(),
                                        title: title.clone(),
                                        icon: icon_image,
                                        is_focused: *is_focused,
                                    };

                                    if idx < win_model.row_count() {
                                        win_model.set_row_data(idx, win_data);
                                    } else {
                                        win_model.push(win_data);
                                    }
                                }
                            }

                            while ws_model.row_count() > paths_to_pass.len() {
                                ws_model.remove(ws_model.row_count() - 1);
                            }

                            for (idx, (ws_id, _)) in paths_to_pass.iter().enumerate() {
                                let win_model = win_map.get(ws_id).unwrap().clone();
                                let ws_data = crate::Workspace {
                                    id: *ws_id,
                                    windows: ModelRc::from(win_model),
                                };

                                if idx < ws_model.row_count() {
                                    ws_model.set_row_data(idx, ws_data);
                                } else {
                                    ws_model.push(ws_data);
                                }
                            }

                            let active_ws_ids: HashSet<i32> = paths_to_pass.iter().map(|(id, _)| *id).collect();
                            win_map.retain(|id, _| active_ws_ids.contains(id));
                        });
                    });
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