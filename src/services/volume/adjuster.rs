use log::{info, error};
use wayle_audio::{volume::types::Volume, AudioService};

use crate::barWindow;
use std::sync::Arc;

pub async fn start_volume_adjuster(
    ui_weak: slint::Weak<barWindow>,
    audio_service: Arc<AudioService>,
    allow_overflow: bool,
) {
    info!("starting volume adjuster and device selector");

    if let Some(ui) = ui_weak.upgrade() {
        let audio_service_vol = Arc::clone(&audio_service);
        let audio_service_mute = Arc::clone(&audio_service);
        let audio_service_in_vol = Arc::clone(&audio_service);
        let audio_service_in_mute = Arc::clone(&audio_service);

        let audio_service_select_out = Arc::clone(&audio_service);
        let audio_service_select_in = Arc::clone(&audio_service);


        ui.on_set_volume(move |volume, delta| {
            let volume_calc = if allow_overflow {
                volume + delta
            } else {
                (volume + delta).clamp(0, 100)
            };
            let normalized = volume_calc as f64 / 100.0;

            let audio_service = Arc::clone(&audio_service_vol);

            tokio::spawn(async move {
                if let Some(device) = audio_service.default_output.get() {
                    let _ = device
                        .set_volume(Volume::stereo(normalized, normalized))
                        .await;
                }
            });
        });

        ui.on_set_muted(move |muted| {
            let audio_service = Arc::clone(&audio_service_mute);

            tokio::spawn(async move {
                if let Some(device) = audio_service.default_output.get() {
                    let _ = device
                        .set_mute(muted)
                        .await;
                }
            });
        });

        ui.on_set_input_volume(move |volume, delta| {
            let volume_calc = if allow_overflow {
                volume + delta
            } else {
                (volume + delta).clamp(0, 100)
            };
            let normalized = volume_calc as f64 / 100.0;

            let audio_service = Arc::clone(&audio_service_in_vol);

            tokio::spawn(async move {
                if let Some(device) = audio_service.default_input.get() {
                    let _ = device
                        .set_volume(Volume::stereo(normalized, normalized))
                        .await;
                }
            });
        });

        ui.on_set_input_muted(move |muted| {
            let audio_service = Arc::clone(&audio_service_in_mute);
info!("help");

            tokio::spawn(async move {
                if let Some(device) = audio_service.default_input.get() {
                    let _ = device
                        .set_mute(muted)
                        .await;
                }
            });
        });

        ui.on_select_output_device(move |id| {
            let audio_service = Arc::clone(&audio_service_select_out);
            let target_desc = id.to_string();
info!("help");

            tokio::spawn(async move {
                let outputs = audio_service.output_devices.get();
                if let Some(device) = outputs.iter().find(|d| d.name.get() == target_desc) {
                    if let Err(e) = device.set_as_default().await {
                        error!("Failed to select default output device: {:?}", e);
                    }
                }
            });
        });

        ui.on_select_input_device(move |id| {
            let audio_service = Arc::clone(&audio_service_select_in);
            let target_desc = id.to_string();
info!("help");
            tokio::spawn(async move {
                let inputs = audio_service.input_devices.get();
                if let Some(device) = inputs.iter().find(|d| d.name.get() == target_desc) {
                    if let Err(e) = device.set_as_default().await {
                        error!("Failed to select default input device: {:?}", e);
                    }
                }
            });
        });
    }
}