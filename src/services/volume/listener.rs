use futures::StreamExt;
use log::{info, error};
use slint::{Model, ModelRc, ToSharedString, VecModel};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use wayle_audio::AudioService;

use crate::{barWindow, VolumeDataSlint, AudioDeviceSlint};

thread_local! {
    static OUTPUT_DEVICES_MODEL: RefCell<Rc<VecModel<AudioDeviceSlint>>> = RefCell::new(Rc::new(VecModel::default()));
    static INPUT_DEVICES_MODEL: RefCell<Rc<VecModel<AudioDeviceSlint>>> = RefCell::new(Rc::new(VecModel::default()));
}

fn update_output_ui(ui_weak: &slint::Weak<barWindow>, volume: i32, muted: bool) {
    let ui = ui_weak.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui.upgrade() {
            ui.set_volumeData(VolumeDataSlint { volume, muted });
        }
    });
}

fn update_input_ui(ui_weak: &slint::Weak<barWindow>, volume: i32, muted: bool) {
    let ui = ui_weak.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui.upgrade() {
            ui.set_inputVolumeData(VolumeDataSlint { volume, muted });
        }
    });
}

fn ensure_models_bound(ui: &barWindow) {
    let current_output_model = ui.get_output_devices();
    if current_output_model.clone().as_any().downcast_ref::<VecModel<AudioDeviceSlint>>().is_none() {
        OUTPUT_DEVICES_MODEL.with(|cell| {
            ui.set_output_devices(ModelRc::from(cell.borrow().clone()));
        });
    }

    let current_input_model = ui.get_input_devices();
    if current_input_model.clone().as_any().downcast_ref::<VecModel<AudioDeviceSlint>>().is_none() {
        INPUT_DEVICES_MODEL.with(|cell| {
            ui.set_input_devices(ModelRc::from(cell.borrow().clone()));
        });
    }
}

fn update_device_lists(ui_weak: &slint::Weak<barWindow>, audio_service: &AudioService) {
    let ui_weak = ui_weak.clone();
    let outputs = audio_service.output_devices.get();
    let inputs = audio_service.input_devices.get();
    
    let default_output_desc = audio_service.default_output.get()
        .map(|d| d.description.get())
        .unwrap_or_default();
        
    let default_input_desc = audio_service.default_input.get()
        .map(|d| d.description.get())
        .unwrap_or_default();

    let slint_outputs: Vec<AudioDeviceSlint> = outputs.iter().map(|dev| {
        let name = dev.name.get();
        let desc = dev.description.get();
        AudioDeviceSlint {
            id: name.to_shared_string(),
            name: desc.clone().to_shared_string(),
            is_default: desc == default_output_desc,
        }
    }).collect();

    let slint_inputs: Vec<AudioDeviceSlint> = inputs.iter()
        .filter(|dev| {
            let name = dev.name.get().to_lowercase();
            let desc = dev.description.get().to_lowercase();

            if name.contains(".monitor") || desc.contains("monitor") {
                return false;
            }

            if name.contains("dummy") || name.contains("null") {
                return false;
            }

            if name.trim().is_empty() || desc.trim().is_empty() {
                return false;
            }

            true
        })
        .map(|dev| {
            let name = dev.name.get();
            let desc = dev.description.get();
            AudioDeviceSlint {
                id: name.to_shared_string(),
                name: desc.clone().to_shared_string(),
                is_default: desc == default_input_desc,
            }
        })
        .collect();

    let default_out_shared = default_output_desc.to_shared_string();
    let default_in_shared = default_input_desc.to_shared_string();

    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ensure_models_bound(&ui);

            crate::helpers::slint_vector::vector::update_vec_model(
                &OUTPUT_DEVICES_MODEL,
                &slint_outputs,
                |item| item.clone(),
            );

            crate::helpers::slint_vector::vector::update_vec_model(
                &INPUT_DEVICES_MODEL,
                &slint_inputs,
                |item| item.clone(),
            );

            ui.set_current_output_name(default_out_shared);
            ui.set_current_input_name(default_in_shared);
        }
    });
}

pub async fn listen_audio_changes(
    ui_weak: slint::Weak<barWindow>, 
    audio_service: Arc<AudioService>,
) {
    info!("Starting audio state listeners");

    let init_ui = ui_weak.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = init_ui.upgrade() {
            ensure_models_bound(&ui);
        }
    });

    update_device_lists(&ui_weak, &audio_service);

    let ui_out = ui_weak.clone();
    let audio_out = audio_service.clone();
    tokio::spawn(async move {
        let mut default_out_stream = audio_out.default_output.watch();
        loop {
            let Some(device) = audio_out.default_output.get() else {
                update_output_ui(&ui_out, 0, false);
                if default_out_stream.next().await.is_none() { return; }
                continue;
            };

            update_device_lists(&ui_out, &audio_out);

            update_output_ui(
                &ui_out,
                device.volume.get().average_percentage().round() as i32,
                device.muted.get(),
            );

            let mut volume_stream = device.volume.watch();
            let mut muted_stream = device.muted.watch();

            loop {
                tokio::select! {
                    biased;
                    changed = default_out_stream.next() => {
                        update_device_lists(&ui_out, &audio_out);
                        if changed.is_none() { return; }
                        break; 
                    }
                    val = volume_stream.next() => {
                        if let Some(v) = val {
                            update_output_ui(&ui_out, v.average_percentage().round() as i32, device.muted.get());
                        } else { break; }
                    }
                    mut_state = muted_stream.next() => {
                        if let Some(m) = mut_state {
                            update_output_ui(&ui_out, device.volume.get().average_percentage().round() as i32, m);
                        } else { break; }
                    }
                }
            }
        }
    });

    let ui_in = ui_weak.clone();
    let audio_in = audio_service.clone();
    tokio::spawn(async move {
        let mut default_in_stream = audio_in.default_input.watch();
        loop {
            let Some(device) = audio_in.default_input.get() else {
                update_input_ui(&ui_in, 0, false);
                if default_in_stream.next().await.is_none() { return; }
                continue;
            };

            update_device_lists(&ui_in, &audio_in);

            update_input_ui(
                &ui_in,
                device.volume.get().average_percentage().round() as i32,
                device.muted.get(),
            );

            let mut volume_stream = device.volume.watch();
            let mut muted_stream = device.muted.watch();

            loop {
                tokio::select! {
                    biased;
                    changed = default_in_stream.next() => {
                        update_device_lists(&ui_in, &audio_in);
                        if changed.is_none() { return; }
                        break;
                    }
                    val = volume_stream.next() => {
                        if let Some(v) = val {
                            update_input_ui(&ui_in, v.average_percentage().round() as i32, device.muted.get());
                        } else { break; }
                    }
                    mut_state = muted_stream.next() => {
                        if let Some(m) = mut_state {
                            update_input_ui(&ui_in, device.volume.get().average_percentage().round() as i32, m);
                        } else { break; }
                    }
                }
            }
        }
    });

    let ui_list = ui_weak.clone();
    let audio_list = audio_service.clone();
    tokio::spawn(async move {
        let mut outputs_changed = audio_list.output_devices.watch();
        let mut inputs_changed = audio_list.input_devices.watch();

        loop {
            tokio::select! {
                _ = outputs_changed.next() => update_device_lists(&ui_list, &audio_list),
                _ = inputs_changed.next() => update_device_lists(&ui_list, &audio_list),
            }
        }
    });
}