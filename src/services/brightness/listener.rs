use log::{error, info};
use notify::event::{CreateKind, ModifyKind, RemoveKind};
use notify::{recommended_watcher, EventKind, RecursiveMode, Watcher};
use std::path::PathBuf;

use crate::barWindow;

async fn get_brightness_status(device_name: &str) -> u32 {
    let brightness_path = format!("/sys/class/backlight/{}/brightness", device_name);
    let max_brightness_path = format!("/sys/class/backlight/{}/max_brightness", device_name);

    let brightness = tokio::fs::read_to_string(&brightness_path)
        .await
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
        .unwrap_or(0);

    let max_brightness = tokio::fs::read_to_string(&max_brightness_path)
        .await
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
        .unwrap_or(100);

    ((brightness as f32 / max_brightness as f32) * 100.0) as u32
}

pub async fn listen_brightness_changes(
    config: &crate::config::AppConfig,
    ui_weak: slint::Weak<barWindow>,
) {
    info!("starting brightness listener");
    let brightness_device = config.config.hardware_config.brightness_device.clone();

    tokio::spawn(async move {
        let device_path = PathBuf::from(format!("/sys/class/backlight/{}", brightness_device));
        
        let initial_brightness = get_brightness_status(&brightness_device).await;
        let ui_init = ui_weak.clone();
        
        slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_init.upgrade() {
                let brightness = initial_brightness as i32;
                ui.set_brightness(brightness);
            }
        })
        .unwrap_or_default();

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        let mut watcher = recommended_watcher(move |res| {
            let _ = tx.send(res);
        })
        .unwrap();

        let mut watcher_path = device_path.clone();
        watcher_path.push("brightness");

        if watcher_path.exists() {
            watcher
                .watch(&watcher_path, RecursiveMode::Recursive)
                .unwrap();
        }

        while let Some(res) = rx.recv().await {
            match res {
                Ok(event) => match event.kind {
                    EventKind::Modify(ModifyKind::Data(_))
                    | EventKind::Create(CreateKind::Any)
                    | EventKind::Remove(RemoveKind::Any) => {
                        let brightness = get_brightness_status(&brightness_device).await;
                        let ui = ui_weak.clone();
                        
                        slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui.upgrade() {
                                let brightness = brightness as i32;
                                ui.set_brightness(brightness);
                            }
                        })
                        .unwrap_or_default();
                    }
                    _ => {}
                },
                Err(e) => error!("watch error: {:?}", e),
            }
        }
    });
}