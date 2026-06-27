use crate::barWindow;

#[cfg(feature = "framework")]
pub(crate) mod framework;
#[cfg(feature = "framework")]
use crate::services::hardware_specific::framework::fans::{adjuster::start_fan_profile_adjuster_framework, listener::listen_fan_profile_changes_framework};

pub async fn harware_specific_management(
    config: &crate::config::AppConfig,
    ui_weak: slint::Weak<barWindow>,
) {
    #[cfg(feature = "framework")]
    listen_fan_profile_changes_framework(&config, ui_weak.clone()).await;
    #[cfg(feature = "framework")]
    start_fan_profile_adjuster_framework(&config, ui_weak.clone()).await;
}
