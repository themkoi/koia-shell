use futures::StreamExt;
use log::{info, error};
use serde::Deserialize;
use tokio::process::Command;
use tokio_util::codec::{FramedRead, LinesCodec};
use crate::barWindow;

#[derive(Deserialize, Debug)]
struct StasisStatus {
    manually_paused: bool,
}

pub async fn listen_idle_changes(ui_weak: slint::Weak<barWindow>) {
    info!("starting idle management listener via stasis");

    tokio::spawn(async move {
        let mut child = match Command::new("stasis")
            .arg("watch")
            .stdout(std::process::Stdio::piped())
            .spawn() 
        {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to start 'stasis watch': {}", e);
                return;
            }
        };

        let stdout = child.stdout.take().expect("Failed to open stdout");
        let mut reader = FramedRead::new(stdout, LinesCodec::new());

        while let Some(line_result) = reader.next().await {
            match line_result {
                Ok(line) => {
                    if let Ok(status) = serde_json::from_str::<StasisStatus>(&line) {
                        let ui_update = ui_weak.clone();
                        let is_caffeine_active = status.manually_paused;

                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_update.upgrade() {
                                let mut current_data = ui.get_sessionData();
                                current_data.caffeine = is_caffeine_active;
                                ui.set_sessionData(current_data);
                            }
                        });
                    }
                }
                Err(e) => {
                    error!("Error reading line from stasis watch: {}", e);
                    break;
                }
            }
        }

        let _ = child.wait().await;
    });
}

pub async fn start_caffeine_adjuster(ui_weak: slint::Weak<barWindow>) {
    info!("registering caffeine adjuster callback");

   slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            
            ui.on_set_caffeine(move |enable| {
                info!("Changing caffeine state to: {}", enable);
                
                tokio::spawn(async move {
                    let action = if enable { "pause" } else { "resume" };
                    
                    match Command::new("stasis").arg(action).status().await {
                        Ok(status) if status.success() => {
                            info!("Successfully set stasis state to {}", action);
                        }
                        Ok(status) => {
                            error!("stasis exited with error status: {}", status);
                        }
                        Err(e) => {
                            error!("Failed to execute stasis command: {}", e);
                        }
                    }
                });
            });
        }
    }).unwrap();
}