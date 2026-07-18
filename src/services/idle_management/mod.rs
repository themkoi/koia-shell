use crate::barWindow;

#[cfg(feature = "stasis")]
pub(crate) mod stasis;
#[cfg(feature = "stasis")]
use crate::services::idle_management::stasis::stasis::{listen_idle_changes, start_caffeine_adjuster};

pub async fn idle_management(ui_weak: slint::Weak<barWindow>) {
    #[cfg(feature = "stasis")]
    listen_idle_changes(ui_weak.clone()).await;
    #[cfg(feature = "stasis")]
    start_caffeine_adjuster(ui_weak.clone()).await;
}
