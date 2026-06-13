use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use crate::barWindow;

fn get_volume_status() -> (i32, bool) {
    let output = Command::new("pamixer")
        .arg("--get-volume-human")
        .output()
        .unwrap();

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout_str.trim();

    if trimmed == "muted" {
        let vol_output = Command::new("pamixer")
            .arg("--get-volume")
            .output()
            .unwrap();
        let vol_str = String::from_utf8_lossy(&vol_output.stdout);
        let vol = vol_str.trim().parse::<i32>().unwrap_or(0);
        
        (vol, true)
    } else {
        let vol = trimmed.trim_end_matches('%').parse::<i32>().unwrap_or(0);
        (vol, false)
    }
}

pub fn listen_volume_changes(ui_weak: slint::Weak<barWindow>) {
    std::thread::spawn(move || {
        let (initial_vol, initial_mute) = get_volume_status();
        let ui_init = ui_weak.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_init.upgrade() {
                let mut data = ui.get_data();
                data.volumeData.volume = initial_vol;
                data.volumeData.muted = initial_mute;
                ui.set_data(data);
            }
        });

        let mut pactl = Command::new("pactl")
            .arg("subscribe")
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let stdout = pactl.stdout.take().unwrap();
        let reader = BufReader::new(stdout);

        for line in reader.lines() {
            if let Ok(line) = line {
                if line.contains("on sink") {
                    let (current_vol, current_mute) = get_volume_status();
                    let ui_update = ui_weak.clone();
                    
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_update.upgrade() {
                            let mut data = ui.get_data();
                            
                            if data.volumeData.volume != current_vol || data.volumeData.muted != current_mute {
                                data.volumeData.volume = current_vol;
                                data.volumeData.muted = current_mute;
                                ui.set_data(data);
                            }
                        }
                    });
                }
            }
        }
    });
}