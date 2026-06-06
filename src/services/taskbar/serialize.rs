use crate::config_shell::components::taskbar::SortingMode;
use crate::services::taskbar::cache::{save_cache, set_path, CacheMap};
use crate::services::taskbar::taskbar::State;
use freedesktop_desktop_entry::{default_paths, get_languages_from_env, DesktopEntry, Iter};
use freedesktop_icons::lookup;
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::LazyLock,
};

#[derive(Clone)]
pub struct SerializeState {
    pub workspaces: Vec<Workspace>,
}

#[derive(Clone)]
pub struct Workspace {
    pub id: i32,
    pub windows: Vec<Window>,
}

#[derive(Clone)]
pub struct Window {
    pub id: i32,
    pub app_id: slint::SharedString,
    pub title: slint::SharedString,
    pub icon_path: slint::SharedString,
    pub is_focused: bool,
}

static DESKTOP_ICON_INDEX: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    let locales = get_languages_from_env();
    let mut map = HashMap::new();
    for path in Iter::new(default_paths()) {
        if let Ok(entry) = DesktopEntry::from_path(path.clone(), Some(&locales)) {
            if let Some(icon) = entry.icon() {
                if let Some(stem) = path.file_stem() {
                    map.insert(stem.to_string_lossy().to_lowercase(), icon.to_string());
                }
                if let Some(wm) = entry.startup_wm_class() {
                    map.insert(wm.to_lowercase(), icon.to_string());
                }
            }
        }
    }
    map
});

#[derive(Clone)]
pub struct WorkspaceDraft {
    pub id: i32,
    pub windows: Vec<WindowDraft>,
}

#[derive(Clone)]
pub struct WindowDraft {
    pub id: i32,
    pub app_id: String,
    pub title: String,
    pub icon_path: String,
    pub is_focused: bool,
}

pub fn get_icon_desktop_fallback(app_id: &str, icon_theme: &str, icon_size: u16) -> Option<String> {
    let icon_name = DESKTOP_ICON_INDEX.get(&app_id.to_lowercase())?;
    lookup(icon_name)
        .with_theme(icon_theme)
        .with_size(icon_size)
        .with_cache()
        .find()
        .map(|p| p.to_string_lossy().into_owned())
}

pub fn group_windows(state: &State) -> Vec<WorkspaceDraft> {
    let mut workspaces: HashMap<i32, Vec<WindowDraft>> = HashMap::new();

    for win in &state.windows {
        let ws_id = win.workspace_id.unwrap_or(0) as i32;
        let entry = workspaces.entry(ws_id).or_default();

        entry.push(WindowDraft {
            id: win.id as i32,
            app_id: win.app_id.clone().unwrap_or_default(),
            title: win.title.clone().unwrap_or_default(),
            icon_path: String::new(), // Fast pipeline execution remains IO-free
            is_focused: win.is_focused,
        });
    }

    let mut result: Vec<WorkspaceDraft> = workspaces
        .into_iter()
        .map(|(id, windows)| WorkspaceDraft { id, windows })
        .collect();

    result.sort_by_key(|w| w.id);
    result
}

impl SerializeState {
    pub fn from_parts(
        state: &State,
        icon_size: &u16,
        icon_theme: &String,
        separate_workspaces: &bool,
        sorting_mode: &SortingMode,
        icon_cache: &mut CacheMap,
        check_cache_validity: &bool,
    ) -> Self {
        let mut resolved: HashMap<String, String> = HashMap::new();
        let mut workspaces: HashMap<u64, Workspace> = HashMap::new();

        for win in &state.windows {
            let key = win
                .app_id
                .clone()
                .unwrap_or_else(|| "application-default-icon".into());

            let icon = resolved.entry(key.clone()).or_insert_with(|| {
                let mut icon_path = String::new();

                if let Some(cache) = icon_cache.get(&key) {
                    icon_path = cache.icon_path.clone();
                    if *check_cache_validity && Path::new(&icon_path).exists() {
                        return icon_path;
                    }
                }

                let mut result = lookup(&key)
                    .with_cache()
                    .with_size(*icon_size)
                    .with_theme(icon_theme)
                    .find();

                icon_path = result.unwrap_or_default().to_string_lossy().into_owned();

                if icon_path.is_empty() {
                    let lower = key.to_lowercase();
                    result = lookup(&lower)
                        .with_cache()
                        .with_size(*icon_size)
                        .with_theme(icon_theme)
                        .find();

                    icon_path = result.unwrap_or_default().to_string_lossy().into_owned();
                }

                if icon_path.is_empty() {
                    icon_path = get_icon_desktop_fallback(&key, icon_theme, *icon_size)
                        .unwrap_or_default();
                }

                icon_path
            }).clone();

            let ws_id = if *separate_workspaces {
                win.workspace_id.unwrap_or(0)
            } else {
                0
            };

            let workspace = workspaces.entry(ws_id).or_insert_with(|| Workspace {
                id: ws_id as i32,
                windows: Vec::new(),
            });

            workspace.windows.push(Window {
                id: win.id as i32,
                app_id: key.into(),
                title: win.title.clone().unwrap_or_default().into(),
                icon_path: icon.into(),
                is_focused: win.is_focused,
            });
        }

        let mut ws: Vec<_> = workspaces.into_values().collect();
        ws.sort_by_key(|w| w.id);

        for w in &mut ws {
            match sorting_mode {
                SortingMode::AZ => w.windows.sort_by(|a, b| a.app_id.cmp(&b.app_id)),
                SortingMode::Id => w.windows.sort_by_key(|w| w.id),
                SortingMode::Default => {}
            }
        }

        Self { workspaces: ws }
    }
}