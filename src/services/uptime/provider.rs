use log::{info, error};
use std::time::Duration;

use crate::{barWindow, UpTimeSlint};

async fn get_system_uptime() -> u64 {
    match tokio::fs::read_to_string("/proc/uptime").await {
        Ok(contents) => contents
            .split_whitespace()
            .next()
            .and_then(|sec_str| sec_str.parse::<f64>().ok())
            .map(|secs| secs as u64)
            .unwrap_or(0),
        Err(e) => {
            error!("Failed to read /proc/uptime: {}", e);
            0
        }
    }
}

fn update_uptime_ui(ui_weak: &slint::Weak<barWindow>, total_seconds: u64) {
    let ui_weak = ui_weak.clone();
    
    let hour = (total_seconds / 3600) as i32;
    let minute = ((total_seconds % 3600) / 60) as i32;

    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_upTimeData(UpTimeSlint { hour, minute });
        }
    });
}

pub async fn provide_uptime(ui_weak: slint::Weak<barWindow>) {
    info!("Starting tokio system uptime worker");

    tokio::spawn(async move {
        // Initial immediate update
        update_uptime_ui(&ui_weak, get_system_uptime().await);

        let mut interval = tokio::time::interval(Duration::from_secs(60));
        interval.tick().await; 

        loop {
            interval.tick().await;
            update_uptime_ui(&ui_weak, get_system_uptime().await);
        }
    });
}