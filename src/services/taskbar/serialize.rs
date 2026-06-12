use std::{
    collections::HashMap,
    path::Path,
};

use crate::config_shell::components::taskbar::SortingMode;
use crate::services::taskbar::cache::{CacheMap, save_cache, set_path};
use crate::services::taskbar::taskbar::State;

use freedesktop_desktop_entry::{
    default_paths, get_languages_from_env, DesktopEntry, Iter,
};
use freedesktop_icons::lookup;

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

fn build_desktop_icon_index() -> HashMap<String, String> {
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
}

pub fn get_icon_desktop_fallback(
    app_id: &str,
    icon_theme: &str,
    icon_size: u16,
    desktop_icon_index: &HashMap<String, String>,
) -> Option<String> {
    let icon_name = desktop_icon_index.get(&app_id.to_lowercase())?;

    lookup(icon_name)
        .with_theme(icon_theme)
        .with_size(icon_size)
        .with_cache()
        .find()
        .map(|p| p.to_string_lossy().into_owned())
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
        let mut cache_changed = false;

        let desktop_icon_index = build_desktop_icon_index();

        let mut resolved: HashMap<String, String> = HashMap::new();

        let unique_apps: Vec<String> = state
            .windows
            .iter()
            .map(|w| {
                w.app_id
                    .clone()
                    .unwrap_or_else(|| "application-default-icon".into())
            })
            .collect();

        let results: Vec<(String, String)> = unique_apps
            .into_iter()
            .map(|app_id| {
                let key = app_id.clone();
                let mut icon_path = String::new();
                let mut run_lookup = true;

                if let Some(cache) = icon_cache.get(&key) {
                    icon_path = cache.icon_path.clone();

                    if *check_cache_validity && Path::new(&icon_path).exists() {
                        run_lookup = false;
                    }
                }

                if run_lookup {
                    let mut icon = lookup(&key)
                        .with_cache()
                        .with_size(*icon_size)
                        .with_theme(icon_theme)
                        .find();

                    icon_path = icon
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned();

                    if icon_path.is_empty() {
                        let lower = key.to_lowercase();

                        icon = lookup(&lower)
                            .with_cache()
                            .with_size(*icon_size)
                            .with_theme(icon_theme)
                            .find();

                        icon_path = icon
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned();
                    }

                    if icon_path.is_empty() {
                        icon_path = get_icon_desktop_fallback(
                            &key,
                            icon_theme,
                            *icon_size,
                            &desktop_icon_index,
                        )
                        .unwrap_or_default();
                    }

                    if icon_path.is_empty() {
                        icon = lookup("application-x-executable")
                            .with_cache()
                            .with_size(*icon_size)
                            .with_theme(icon_theme)
                            .find();

                        icon_path = icon
                            .unwrap_or_default()
                            .to_string_lossy()
                            .into_owned();
                    }
                }

                (key, icon_path)
            })
            .collect();

        for (key, path) in &results {
            if !path.is_empty() {
                set_path(icon_cache, key, path);
                cache_changed = true;
            }

            resolved.insert(key.clone(), path.clone());
        }

        let mut workspaces_map =
            std::collections::BTreeMap::<u64, Workspace>::new();

        for win in &state.windows {
            let key = win
                .app_id
                .clone()
                .unwrap_or_else(|| "application-default-icon".into());

            let icon_path = resolved.get(&key).cloned().unwrap_or_default();

            let window = Window {
                id: win.id as i32,
                app_id: key.clone().into(),
                title: win.title.clone().unwrap_or_default().into(),
                icon_path: icon_path.into(),
                is_focused: win.is_focused,
            };

            let ws_id = if *separate_workspaces {
                win.workspace_id.unwrap_or(0)
            } else {
                0
            };

            workspaces_map
                .entry(ws_id)
                .or_insert_with(|| Workspace {
                    id: ws_id as i32,
                    windows: Vec::new(),
                })
                .windows
                .push(window);
        }

        let mut workspaces: Vec<Workspace> =
            workspaces_map.into_values().collect();

        workspaces.sort_by_key(|ws| ws.id);

        for ws in &mut workspaces {
            match sorting_mode {
                SortingMode::Default => {}
                SortingMode::AZ => {
                    ws.windows.sort_by(|a, b| a.app_id.cmp(&b.app_id))
                }
                SortingMode::Id => {
                    ws.windows.sort_by_key(|w| w.id)
                }
            }
        }

        if cache_changed {
            save_cache(icon_cache);
        }

        SerializeState { workspaces }
    }
}