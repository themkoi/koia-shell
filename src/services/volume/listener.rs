use futures::StreamExt;
use log::info;
use std::sync::Arc;
use wayle_audio::AudioService;

use crate::{barWindow, VolumeDataSlint};

fn update_ui(
    ui_weak: &slint::Weak<barWindow>,
    volume: i32,
    muted: bool,
) {
    let ui = ui_weak.clone();

    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui.upgrade() {
            ui.set_volumeData(VolumeDataSlint { volume, muted });
        }
    });
}

pub async fn listen_volume_changes(
    ui_weak: slint::Weak<barWindow>,
    audio_service: Arc<AudioService>,
) {
    info!("starting volume listener");

    tokio::spawn(async move {
        let mut output_stream = audio_service.default_output.watch();

        while let Some(maybe_device) = output_stream.next().await {
            let Some(device) = maybe_device else {
                update_ui(&ui_weak, 0, false);
                continue;
            };

            update_ui(
                &ui_weak,
                device.volume.get().average_percentage().round() as i32,
                device.muted.get(),
            );

            let mut volume_stream = device.volume.watch();
            let mut muted_stream = device.muted.watch();

            loop {
                tokio::select! {
                    volume = volume_stream.next() => {
                        let Some(volume) = volume else {
                            break;
                        };

                        update_ui(
                            &ui_weak,
                            volume.average_percentage().round() as i32,
                            device.muted.get(),
                        );
                    }

                    muted = muted_stream.next() => {
                        let Some(muted) = muted else {
                            break;
                        };

                        update_ui(
                            &ui_weak,
                            device.volume.get().average_percentage().round() as i32,
                            muted,
                        );
                    }

                    changed = output_stream.next() => {
                        match changed {
                            Some(_) => break,
                            None => return,
                        }
                    }
                }
            }
        }
    });
}