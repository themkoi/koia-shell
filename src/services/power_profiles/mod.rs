use crate::{
    barWindow,
    services::power_profiles::{
        adjuster::start_profile_adjuster, listener::listen_profile_changes,
    },
};
use wayle_power_profiles::PowerProfilesService;

mod adjuster;
mod listener;

pub async fn start_power_profile_management(ui_weak: slint::Weak<barWindow>) {
    tokio::spawn(async move {
        let profile = PowerProfilesService::new().await.unwrap();

        listen_profile_changes(ui_weak.clone(), profile.clone()).await;
        start_profile_adjuster(ui_weak, profile).await;
    });
}
