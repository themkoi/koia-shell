use crate::{
    barWindow,
    services::power_profiles::{
        adjuster::start_profile_adjuster, listener::listen_profile_changes,
    },
};
use wayle_power_profiles::PowerProfilesService;

mod adjuster;
mod listener;

use tokio::sync::oneshot;

pub async fn start_power_profile_management(ui_weak: slint::Weak<barWindow>) {
    let (tx, rx) = oneshot::channel();

    tokio::spawn(async move {
        let service = PowerProfilesService::new().await.unwrap();
        tx.send(service).unwrap();
    });

    let profile = rx.await.unwrap();

    start_profile_adjuster(ui_weak.clone(), profile.clone()).await;

    listen_profile_changes(ui_weak, profile).await;
}
