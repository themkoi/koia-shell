use log::info;
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

            tokio::spawn(async move {
                let _ = profile_service
                    .power_profiles
                    .set_active_profile(profile.to_string().as_str().into())
                    .await;
            });
        });
    }
}
