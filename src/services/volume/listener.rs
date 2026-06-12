use libpulse_binding as pulse;

use log::info;
use pulse::callbacks::ListResult;
use pulse::context::subscribe::{Facility, InterestMaskSet, Operation};
use pulse::context::{Context, FlagSet};
use pulse::mainloop::standard::Mainloop;
use pulse::proplist::Proplist;
use pulse::volume::Volume;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::{barWindow, DataSlint};

#[derive(Debug, Default)]
pub struct VolumeState {
    pub sink_volume: Option<Volume>,
    pub volume_percent: Option<f32>,
}

pub fn listen_volume_changes(ui_weak: slint::Weak<barWindow>) {
    let state = Arc::new(Mutex::new(VolumeState::default()));
    let state_cloned = state.clone();
    let ui_weak_outer = ui_weak.clone();

    std::thread::spawn(move || {
        let mut mainloop = Mainloop::new().unwrap();

        let mut proplist = Proplist::new().unwrap();
        proplist
            .set_str(
                pulse::proplist::properties::APPLICATION_NAME,
                "rust-volume-listener",
            )
            .unwrap();

        let context = Context::new_with_proplist(&mainloop, "VolumeListener", &proplist).unwrap();
        let context = Rc::new(RefCell::new(context));

        context
            .borrow_mut()
            .connect(None, FlagSet::NOFLAGS, None)
            .unwrap();

        let ctx_sub = Rc::clone(&context);
        let state_inner = state_cloned.clone();

        context.borrow_mut().set_subscribe_callback(Some(Box::new(
            move |facility, operation, index| {
                if facility == Some(Facility::Sink) && operation == Some(Operation::Changed) {
                    let state2 = state_inner.clone();
                    let ui_weak_inner = ui_weak_outer.clone();

                    ctx_sub
                        .borrow()
                        .introspect()
                        .get_sink_info_by_index(index, move |res| {
                            if let ListResult::Item(sink) = res {
                                let vol = sink.volume.avg();
                                let volume_percent =
                                    (vol.0 as f32 / pulse::volume::Volume::NORMAL.0 as f32) * 100.0;

                                let ui_weak_slint = ui_weak_inner.clone();
                                let _ = slint::invoke_from_event_loop(move || {
                                    if let Some(ui) = ui_weak_slint.upgrade() {
                                        let current_data = ui.get_data();
                                        let new_data = DataSlint {
                                            volume: volume_percent as i32,
                                            ..current_data
                                        };
                                        ui.set_data(new_data);
                                    }
                                });

                                info!("Volume changed: {:.0}%", volume_percent);

                                let mut s = state2.lock().unwrap();
                                s.sink_volume = Some(vol);
                                s.volume_percent = Some(volume_percent);
                            }
                        });
                }
            },
        )));

        let ctx_state = Rc::clone(&context);
        let ui_weak_startup = ui_weak.clone();
        let state_startup = state_cloned.clone();

        context
            .borrow_mut()
            .set_state_callback(Some(Box::new(move || {
                let is_ready = ctx_state.borrow().get_state() == pulse::context::State::Ready;

                if is_ready {
                    info!("PulseAudio ready");

                    ctx_state
                        .borrow_mut()
                        .subscribe(InterestMaskSet::SINK, |success| {
                            if success {
                                info!("Subscribed to sink events");
                            }
                        });

                    let ui_weak_init = ui_weak_startup.clone();
                    let state_init = state_startup.clone();
                    
                    ctx_state.borrow().introspect().get_sink_info_by_name("@DEFAULT_SINK@", move |res| {
                        if let ListResult::Item(sink) = res {
                            let vol = sink.volume.avg();
                            let volume_percent =
                                (vol.0 as f32 / pulse::volume::Volume::NORMAL.0 as f32) * 100.0;

                            let ui_weak_slint = ui_weak_init.clone();
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(ui) = ui_weak_slint.upgrade() {
                                    let current_data = ui.get_data();
                                    let new_data = DataSlint {
                                        volume: volume_percent as i32,
                                        ..current_data
                                    };
                                    ui.set_data(new_data);
                                }
                            });

                            info!("Initial volume written: {:.0}%", volume_percent);

                            let mut s = state_init.lock().unwrap();
                            s.sink_volume = Some(vol);
                            s.volume_percent = Some(volume_percent);
                        }
                    });
                }
            })));

        loop {
            mainloop.iterate(true);
        }
    });
}