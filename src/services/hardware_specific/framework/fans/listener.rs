use log::{error, info};
use serde::Deserialize;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::barWindow;

#[derive(Deserialize)]
struct FanEvent {
    strategy: String,
    speed: Option<u32>,
    paused: Option<bool>,
}

pub async fn listen_fan_profile_changes_framework(
    config: &crate::config::AppConfig,
    ui_weak: slint::Weak<barWindow>,
) {
    if config.config.hardware_config.hardware_specific_features {
        info!("starting fan profile listener");
        tokio::spawn(async move {
            let mut child = match Command::new("fw-fanctrl-rs")
                .arg("listen")
                .stdout(Stdio::piped())
                .spawn()
            {
                Ok(c) => c,
                Err(e) => {
                    error!("failed to start fw-fanctrl-rs listen: {:?}", e);
                    return;
                }
            };

            let stdout = match child.stdout.take() {
                Some(s) => s,
                None => {
                    error!("no stdout from fw-fanctrl-rs");
                    return;
                }
            };

            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            let mut last_strategy: Option<String> = None;

            while let Ok(Some(line)) = lines.next_line().await {
                let parsed: Result<FanEvent, _> = serde_json::from_str(&line);

                match parsed {
                    Ok(event) => {
                        if last_strategy.as_deref() != Some(&event.strategy) {
                            last_strategy = Some(event.strategy.clone());

                            let ui = ui_weak.clone();
                            let strategy = event.strategy.clone();

                            slint::invoke_from_event_loop(move || {
                                if let Some(ui) = ui.upgrade() {
                                    ui.set_fanProfile(strategy.into());
                                }
                            })
                            .unwrap_or_default();
                        }
                    }
                    Err(e) => {
                        error!("failed to parse fan event: {:?}, line: {}", e, line);
                    }
                }
            }

            error!("fw-fanctrl-rs listen stream ended unexpectedly");
        });
    }
}
