use log::{info, error};
use wayle_power_profiles::PowerProfilesService;

use crate::barWindow;
use std::sync::Arc;

pub async fn start_profile_adjuster(
    ui_weak: slint::Weak<barWindow>,
    profile_service: Arc<PowerProfilesService>,
) {
    info!("starting power profile adjuster");

    if let Some(ui) = ui_weak.upgrade() {
        ui.on_set_power_profile(move |profile| {
            let profile_service = Arc::clone(&profile_service);
            let profile_str = profile.to_string(); 
            info!("Received power profile change request from UI: '{}'", profile_str);

            tokio::spawn(async move {
                let target_profile = wayle_power_profiles::types::profile::PowerProfile::from(
                    profile_str.as_str()
                );

                match profile_service
                    .power_profiles
                    .set_active_profile(target_profile.clone())
                    .await 
                {
                    Ok(_) => info!("Successfully updated system power profile to {:?}", target_profile),
                    Err(e) => error!("Failed to set active power profile: {:?}", e),
                }
            });
        });
    }
}