use log::info;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;                  
use std::process::Stdio;

use crate::{barWindow, VolumeDataSlint};

async fn get_volume_status() -> (i32, bool) {
    let output = Command::new("pamixer")
        .arg("--get-volume-human")
        .output()
        .await 
        .unwrap();

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout_str.trim();

    if trimmed == "muted" {
        let vol_output = Command::new("pamixer")
            .arg("--get-volume")
            .output()
            .await
            .unwrap();
        let vol_str = String::from_utf8_lossy(&vol_output.stdout);
        let vol = vol_str.trim().parse::<i32>().unwrap_or(0);

        (vol, true)
    } else {
        let vol = trimmed.trim_end_matches('%').parse::<i32>().unwrap_or(0);
        (vol, false)
    }
}

pub async fn listen_volume_changes(ui_weak: slint::Weak<barWindow>) {
    info!("starting volume listener");
    
    tokio::spawn(async move {
        let (initial_vol, initial_mute) = get_volume_status().await;
        let ui_init = ui_weak.clone();
        
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_init.upgrade() {
                let volume = initial_vol;
                let muted = initial_mute;
                let volume_data = VolumeDataSlint { volume, muted };
                ui.set_volumeData(volume_data);
            }
        });

        let mut pactl = Command::new("pactl")
            .arg("subscribe")
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let stdout = pactl.stdout.take().unwrap();
        
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();

        while let Ok(bytes_read) = reader.read_line(&mut line).await {
            if bytes_read == 0 {
                break; 
            }

            if line.contains("on sink") {
                let (current_vol, current_mute) = get_volume_status().await;
                let ui_update = ui_weak.clone();

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_update.upgrade() {
                        let volume = current_vol;
                        let muted = current_mute;
                        let volume_data = VolumeDataSlint { volume, muted };
                        ui.set_volumeData(volume_data);
                    }
                });
            }
            
            line.clear(); 
        }
    });
}