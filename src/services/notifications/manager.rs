use log::info;
use slint::{ComponentHandle, Image, ModelRc, ToSharedString, VecModel};
use spell_framework::{
    vault::{
        CloseReason, Hint, NotiError, Notification, NotificationManager, Timeout,
        NOTIFICATION_EVENT,
    },
    wayland_adapter::WinHandle,
};
use std::path::Path;
use std::rc::Rc;
use std::sync::OnceLock;

use crate::{notificationWindow, notificationWindowSpell, ActionData};

static CONFIG_CELL: OnceLock<crate::config::AppConfig> = OnceLock::new();

impl NotificationManager for notificationWindow {
    fn new_notification(&self, notification: Notification) -> Result<(), NotiError> {
        info!(
            "Received new notification via framework: {}",
            notification.summary
        );

        let slint_actions_vec = Vec::new();
        let actions_model = Rc::new(VecModel::from(slint_actions_vec));

        let mut chunks = notification.actions.chunks_exact(2);
        while let Some(pair) = chunks.next() {
            actions_model.push(ActionData {
                key: pair[0].to_shared_string(),
                label: pair[1].to_shared_string(),
            });
        }

        if let Some(remainder) = chunks.remainder().first() {
            actions_model.push(ActionData {
                key: remainder.to_shared_string(),
                label: remainder.to_shared_string(),
            });
        }

        let resolved_icon = if let Some(app_config) = CONFIG_CELL.get() {
            resolve_icon(&notification, app_config)
        } else {
            Image::default()
        };

        let mut title = notification.summary.clone();
        let mut body = notification.body.clone();

        if let Some(app_config) = CONFIG_CELL.get() {
            let max_title = app_config.config.notification_config.max_title_lenght as usize;
            let max_text = app_config.config.notification_config.max_text_lenght as usize;

            if title.chars().count() > max_title {
                title = title.chars().take(max_title).collect::<String>() + "...";
            }

            if body.chars().count() > max_text {
                body = body.chars().take(max_text).collect::<String>() + "...";
            }
        }

        self.invoke_add_notif(
            notification.id as i32,
            notification.appname.to_shared_string(),
            title.to_shared_string(), // Passed elided title
            notification.subtitle.unwrap_or_default().to_shared_string(),
            body.to_shared_string(),  // Passed elided body
            give_timeout(notification.timeout),
            resolved_icon,
            ModelRc::from(actions_model),
        );

        Ok(())
    }

    fn close_notification(&self, _id: u32) -> Result<(), NotiError> {
        Ok(())
    }
}

pub async fn start_notification_service(
    config: crate::config::AppConfig,
    ui_spell: &notificationWindowSpell,
) {
    info!("Starting notification window service manager using Tokio async tasks");

    let _ = CONFIG_CELL.set(config);

    let ui_window_weak = ui_spell.ui.as_weak();

    let Some(window) = ui_window_weak.upgrade() else {
        return;
    };

    let loop_handle_clone = ui_spell.way.loop_handle.clone();
    let handle = WinHandle(loop_handle_clone);
    let handlse = handle.clone();

    window.on_a_input_region(move |x, y, width, height| {
        handle.add_input_region(x, y, width, height);
    });

    window.on_r_input_region(move |x, y, width, height| {
        handlse.subtract_input_region(x, y, width, height);
    });

    window.on_noti_close(move |id| {
        tokio::spawn(async move {
            tokio::task::spawn_blocking(move || {
                let _ = NOTIFICATION_EVENT
                    .get()
                    .unwrap()
                    .call_close(id as u32, CloseReason::Dismissed);
            })
            .await
            .ok();
        });
    });

    window.on_noti_action(move |id, action_key| {
        info!("Invoking action for ID: {}, action: {}", id, action_key);

        tokio::spawn(async move {
            let action_key = action_key.to_string();

            tokio::task::spawn_blocking(move || {

                let _ = NOTIFICATION_EVENT
                    .get()
                    .unwrap()
                    .action_invoked(id, &action_key);
            })
            .await
            .ok();
        });
    });
}

fn give_timeout(timeout: Timeout) -> i32 {
    match timeout {
        Timeout::Default => {
            if let Some(cfg) = CONFIG_CELL.get() {
                cfg.config.notification_config.notification_timeout.into()
            } else {
                3000
            }
        }
        Timeout::Never => 0,
        Timeout::Milliseconds(val) => val,
    }
}

fn resolve_icon(notif: &Notification, app_config: &crate::config::AppConfig) -> Image {
    let mut target_string = String::new();
    for hint in &notif.hints {
        if let Hint::ImagePath(path) = hint {
            if !path.is_empty() {
                target_string = path.clone();
                break;
            }
        }
    }

    if target_string.is_empty() {
        target_string = notif.icon.clone();
    }

    if target_string.is_empty() {
        for hint in &notif.hints {
            if let Hint::DesktopEntry(entry) = hint {
                if !entry.is_empty() {
                    target_string = entry.clone();
                    break;
                }
            }
        }
    }

    if target_string.is_empty() {
        return Image::default();
    }

    let path = Path::new(&target_string);
    if path.is_absolute() && path.exists() {
        if let Ok(img) = Image::load_from_path(path) {
            return img;
        }
    } else {
        let target_size = app_config.config.notification_config.icon_size as u16;
        let theme_name = &app_config.config.icon_theme;

        if let Some(icon_path) = freedesktop_icons::lookup(&target_string)
            .with_theme(theme_name)
            .with_size(target_size)
            .find()
        {
            if let Ok(img) = Image::load_from_path(&icon_path) {
                return img;
            }
        }
    }

    Image::default()
}
