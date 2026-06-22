use log::info;
use wayle_audio::{volume::types::Volume, AudioService};

use crate::barWindow;
use std::sync::Arc;

pub async fn start_volume_adjuster(
    ui_weak: slint::Weak<barWindow>,
    audio_service: Arc<AudioService>,
    allow_overflow: bool,
) {
    info!("starting volume adjuster");

    if let Some(ui) = ui_weak.upgrade() {
        let audio_service = Arc::clone(&audio_service);
        let audio_service_on_mute = Arc::clone(&audio_service);

        ui.on_set_volume(move |volume, delta| {
            let volume_calc;
            if allow_overflow == true {
                volume_calc = volume + delta;
            } else {
                volume_calc = (volume + delta).clamp(0, 100);
            }
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

        ui.on_set_muted(move |muted | {
            let audio_service = Arc::clone(&audio_service_on_mute);

            tokio::spawn(async move {
                if let Some(device) = audio_service.default_output.get() {
                    let _ = device
                        .set_mute(muted)
                        .await;
                }
            });
        });
    }
}
