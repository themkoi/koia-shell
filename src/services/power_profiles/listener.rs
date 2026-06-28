use std::sync::Arc;

use crate::barWindow;
use futures::StreamExt;
use log::info;
use slint::ToSharedString;
use wayle_power_profiles::PowerProfilesService;

fn format_profile(profile: impl AsRef<str>) -> String {
    profile
        .as_ref()
        .replace('-', " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub async fn listen_profile_changes(
    ui_weak: slint::Weak<barWindow>,
    profile_service: Arc<PowerProfilesService>,
) {
    info!("starting power profile listener");

    tokio::spawn(async move {
        let ui_init = ui_weak.clone();

        let profile_init: wayle_power_profiles::types::profile::PowerProfile =
            profile_service.power_profiles.active_profile.get();

        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_init.upgrade() {
                ui.set_powerProfile(format_profile(&profile_init.to_string()).to_shared_string());
            }
        });

        let mut stream = profile_service.power_profiles.active_profile.watch();

        while let Some(profile) = stream.next().await {
            let ui_update = ui_weak.clone();

            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_update.upgrade() {
                    ui.set_powerProfile(format_profile(&profile.to_string()).to_shared_string());
                }
            });
        }
    });
}
