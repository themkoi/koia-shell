use crate::{barWindow, ContextMenuActionSlint, ContextMenuDataSlint, TrayItemSlint};
use slint::{Image, ModelRc, Rgba8Pixel, SharedPixelBuffer, VecModel, Weak};
use std::rc::Rc;
use std::sync::Arc;

use system_tray::client::{ActivateRequest, Client};
use system_tray::item::IconPixmap;
use system_tray::menu::{MenuItem, MenuType, ToggleState};

pub async fn start_system_tray(config: &crate::config::AppConfig, ui_weak: Weak<barWindow>) {
    println!("[Tray Manager] Initializing full featured system-tray backend...");

    let client_raw = match Client::new().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[Tray Manager] Connection failure: {}", e);
            return;
        }
    };

    let client = Arc::new(client_raw);
    let mut tray_rx = client.subscribe();
    let ui_weak_clone = ui_weak.clone();

    populate_ui_items(&client, &ui_weak_clone, config);

    if let Some(ui) = ui_weak.upgrade() {
        ui.on_tray_item_clicked({
            let ui_weak = ui_weak.clone();
            let client_clone = Arc::clone(&client);
            let config_clone = config.clone();

            move |compound_id, button_type, button_x| {
                let ui = match ui_weak.upgrade() {
                    Some(val) => val,
                    None => return,
                };

                let parts: Vec<&str> = compound_id.split('|').collect();
                let address = match parts.first() {
                    Some(addr) => addr.to_string(),
                    None => return,
                };

                if button_type.as_str() == "left" {
                    let mut current_data = ui.get_data();
                    current_data.menuData.visible = false;
                    ui.set_data(current_data);

                    let client_exec = Arc::clone(&client_clone);
                    tokio::spawn(async move {
                        let req = ActivateRequest::Default {
                            address,
                            x: 0,
                            y: 0,
                        };
                        if let Err(e) = client_exec.activate(req).await {
                            eprintln!("[Tray Exec] Left click activation failed: {:?}", e);
                        }
                    });
                } else if button_type.as_str() == "right" {
                    let mut current_data = ui.get_data();
                    let mut actions = Vec::new();

                    if let Ok(guard) = client_clone.items().lock() {
                        if let Some((system_item, Some(tray_menu))) = guard.get(&address) {
                            let menu_path = system_item.menu.clone().unwrap_or_else(|| {
                                format!("/org/ayatana/NotificationItem/{}/Menu", system_item.id)
                            });
                            let window_title = system_item
                                .title
                                .clone()
                                .unwrap_or_else(|| "Tray Options".into());

                            flatten_menu_tree(
                                &tray_menu.submenus,
                                &address,
                                &menu_path,
                                0,
                                &mut actions,
                                &config_clone,
                            );
                            let visible: bool;

                            if current_data.menuData.item_id != compound_id {
                                visible = true
                            } else {
                                visible = !current_data.menuData.visible;
                            }

                            current_data.menuData = ContextMenuDataSlint {
                                visible: visible,
                                item_id: compound_id.clone(),
                                title: window_title.into(),
                                x_pos: button_x,
                                actions: ModelRc::from(Rc::new(VecModel::from(actions))),
                            };
                            ui.set_data(current_data);
                        }
                    }
                }
            }
        });

        ui.on_menu_action_executed({
            let ui_weak = ui_weak.clone();
            let client_clone = Arc::clone(&client);

            move |_item_id, action_id| {
                let ui = match ui_weak.upgrade() {
                    Some(val) => val,
                    None => return,
                };

                let action_str = action_id.to_string();
                let parts: Vec<&str> = action_str.split('|').collect();

                if parts.len() == 3 {
                    let address = parts[0].to_string();
                    let menu_path = parts[1].to_string();

                    if let Ok(submenu_id) = parts[2].parse::<i32>() {
                        let client_exec = Arc::clone(&client_clone);

                        tokio::spawn(async move {
                            let req = ActivateRequest::MenuItem {
                                address,
                                menu_path,
                                submenu_id,
                            };
                            if let Err(e) = client_exec.activate(req).await {
                                eprintln!("[Tray Exec] Submenu action selection failed: {:?}", e);
                            }
                        });
                    }
                }

                let mut current_data = ui.get_data();
                current_data.menuData.visible = false;
                ui.set_data(current_data);
            }
        });
    }

    let client_bg = Arc::clone(&client);
    let config_bg = config.clone();
    tokio::spawn(async move {
        while let Ok(_event) = tray_rx.recv().await {
            populate_ui_items(&client_bg, &ui_weak_clone, &config_bg);
        }
    });
}

fn create_slint_image_from_argb(raw_argb: &[u8], width: i32, height: i32) -> Image {
    if raw_argb.is_empty() || width <= 0 || height <= 0 {
        return Image::default();
    }

    let mut buffer = SharedPixelBuffer::<Rgba8Pixel>::new(width as u32, height as u32);
    let buffer_stride = buffer.make_mut_bytes();

    let total_bytes = (width * height * 4) as usize;
    let available_bytes = std::cmp::min(raw_argb.len(), total_bytes);

    for i in (0..available_bytes).step_by(4) {
        if i + 3 < buffer_stride.len() && i + 3 < raw_argb.len() {
            let a = raw_argb[i];
            let r = raw_argb[i + 1];
            let g = raw_argb[i + 2];
            let b = raw_argb[i + 3];

            buffer_stride[i] = r;
            buffer_stride[i + 1] = g;
            buffer_stride[i + 2] = b;
            buffer_stride[i + 3] = a;
        }
    }

    Image::from_rgba8(buffer)
}

fn decode_dbus_menu_icon(encoded_bytes: &[u8]) -> Image {
    if encoded_bytes.len() < 8 {
        return Image::default();
    }

    if let Ok(dynamic_img) = image::load_from_memory(encoded_bytes) {
        let rgba_img = dynamic_img.to_rgba8();
        let (w, h) = rgba_img.dimensions();

        let mut buffer = SharedPixelBuffer::<Rgba8Pixel>::new(w, h);
        buffer.make_mut_bytes().copy_from_slice(rgba_img.as_raw());
        return Image::from_rgba8(buffer);
    }

    let mut width_bytes = [0u8; 4];
    let mut height_bytes = [0u8; 4];
    width_bytes.copy_from_slice(&encoded_bytes[0..4]);
    height_bytes.copy_from_slice(&encoded_bytes[4..8]);

    let width = i32::from_be_bytes(width_bytes);
    let height = i32::from_be_bytes(height_bytes);

    if width > 0 && height > 0 && (encoded_bytes.len() - 8) >= (width * height * 4) as usize {
        return create_slint_image_from_argb(&encoded_bytes[8..], width, height);
    }

    let width_le = i32::from_le_bytes(width_bytes);
    let height_le = i32::from_le_bytes(height_bytes);

    if width_le > 0
        && height_le > 0
        && (encoded_bytes.len() - 8) >= (width_le * height_le * 4) as usize
    {
        return create_slint_image_from_argb(&encoded_bytes[8..], width_le, height_le);
    }

    Image::default()
}

fn create_best_image_from_pixmaps(pixmaps: &[IconPixmap], target_size: i32) -> Image {
    if pixmaps.is_empty() {
        return Image::default();
    }

    let best_pixmap = pixmaps
        .iter()
        .min_by_key(|p| (p.width - target_size).abs())
        .unwrap_or(&pixmaps[0]);

    create_slint_image_from_argb(&best_pixmap.pixels, best_pixmap.width, best_pixmap.height)
}

fn lookup_fallback_theme_icon(
    icon_name: &str,
    target_size: i32,
    config: &crate::config::AppConfig,
) -> Image {
    if icon_name.is_empty() {
        return Image::default();
    }

    if let Some(path_buf) = freedesktop_icons::lookup(icon_name)
        .with_theme(&config.config.icon_theme)
        .with_size(target_size as u16)
        .find()
    {
        if let Ok(slint_img) = Image::load_from_path(&path_buf) {
            return slint_img;
        }
    }
    Image::default()
}

fn flatten_menu_tree(
    menu_items: &[MenuItem],
    address: &str,
    menu_path: &str,
    depth: i32,
    target_list: &mut Vec<ContextMenuActionSlint>,
    config: &crate::config::AppConfig,
) {
    let target_size = config.config.tray_config.menu_icon_size as i32;

    for item in menu_items {
        if !item.visible {
            continue;
        }

        let is_separator = item.menu_type == MenuType::Separator;
        let display_label = item.label.clone().unwrap_or_default();
        let compound_action_id = format!("{}|{}|{}", address, menu_path, item.id);
        let is_destructive =
            item.label.as_deref() == Some("Quit") || item.label.as_deref() == Some("Exit");

        let mut slint_icon = Image::default();

        if let Some(ref name) = item.icon_name {
            slint_icon = lookup_fallback_theme_icon(name, target_size, config);
        }

        if slint_icon.size().width == 0 {
            if let Some(ref encoded_bytes) = item.icon_data {
                slint_icon = decode_dbus_menu_icon(encoded_bytes);
            }
        }

        let has_icon = slint_icon.size().width > 0;

        let toggle_state_flag = match item.toggle_type {
            system_tray::menu::ToggleType::Checkmark => match item.toggle_state {
                ToggleState::On => 2,
                _ => 1,
            },
            system_tray::menu::ToggleType::Radio => match item.toggle_state {
                ToggleState::On => 4,
                _ => 3,
            },
            system_tray::menu::ToggleType::CannotBeToggled => 0,
        };

        target_list.push(ContextMenuActionSlint {
            id: compound_action_id.clone().into(),
            label: display_label.into(),
            icon: slint_icon,
            has_icon,
            is_destructive,
            is_separator,
            toggle_state: toggle_state_flag,
            depth,
        });

        if !item.submenu.is_empty() {
            flatten_menu_tree(
                &item.submenu,
                address,
                menu_path,
                depth + 1,
                target_list,
                config,
            );
        }
    }
}

struct ThreadSafeTrayData {
    id: String,
    title: String,
    status: String,
    active: bool,
    icon_name: Option<String>,
    raw_pixmaps: Option<Vec<IconPixmap>>,
}

fn populate_ui_items(
    client: &Arc<Client>,
    ui_weak: &Weak<barWindow>,
    config: &crate::config::AppConfig,
) {
    let mut thread_safe_items = Vec::new();

    if let Ok(guard) = client.items().lock() {
        for (address, (system_item, _menu_option)) in guard.iter() {
            let status_str = format!("{:?}", system_item.status);
            let is_active = status_str == "NeedsAttention";

            let menu_path = system_item.menu.clone().unwrap_or_else(|| {
                format!("/org/ayatana/NotificationItem/{}/Menu", system_item.id)
            });

            let compound_id = format!("{}|{}|{}", address, menu_path, system_item.id);

            thread_safe_items.push(ThreadSafeTrayData {
                id: compound_id,
                title: system_item.title.clone().unwrap_or_default(),
                status: status_str,
                active: is_active,
                icon_name: system_item.icon_name.clone(),
                raw_pixmaps: system_item.icon_pixmap.clone(),
            });
        }
    }

    let config_clone = config.clone();
    let _ = ui_weak.upgrade_in_event_loop(move |ui| {
        let mut ui_items = Vec::new();
        let target_size = config_clone.config.tray_config.icon_size as i32;

        for raw_item in thread_safe_items {
            let mut tray_icon = Image::default();

            if let Some(ref name) = raw_item.icon_name {
                tray_icon = lookup_fallback_theme_icon(name, target_size, &config_clone);
            }

            if tray_icon.size().width == 0 {
                if let Some(ref pixmaps) = raw_item.raw_pixmaps {
                    tray_icon = create_best_image_from_pixmaps(pixmaps, target_size);
                }
            }

            ui_items.push(TrayItemSlint {
                id: raw_item.id.into(),
                title: raw_item.title.into(),
                icon: tray_icon,
                status: raw_item.status.into(),
                active: raw_item.active,
            });
        }

        let mut d = ui.get_data();
        d.tray_items = ModelRc::from(Rc::new(VecModel::from(ui_items)));
        ui.set_data(d);
    });
}
