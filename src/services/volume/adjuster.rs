use log::info;
use wayle_audio::{volume::types::Volume, AudioService};

use crate::barWindow;
use std::sync::Arc;

pub async fn start_volume_adjuster(
    ui_weak: slint::Weak<barWindow>,
    audio_service: Arc<AudioService>,
) {
    info!("starting volume adjuster");

    if let Some(ui) = ui_weak.upgrade() {
        let audio_service = Arc::clone(&audio_service);

        ui.on_set_volume(move |volume, delta| {
            let volume_calc = volume + delta;
            let normalized = volume_calc as f64 / 100.0;

            let audio_service = Arc::clone(&audio_service);

            tokio::spawn(async move {
                if let Some(device) = audio_service.default_output.get() {
                    let _ = device
                        .set_volume(Volume::stereo(normalized, normalized))
                        .await;
                }
            });
        });
    }
}
