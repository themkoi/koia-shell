use crate::barWindow; // Matching your window's exact naming convention
use ddc_hi::{Ddc, Display};
use log::{debug, error, info, warn};
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Mutex},
    time::Duration,
};

const BRIGHTNESS_CODE: u8 = 0x10;

pub type DisplayId = String;
pub type ScreenBrightness = u16;

#[derive(Debug, Clone)]
pub enum EventToSub {
    Refresh,
    Set(DisplayId, ScreenBrightness),
}

thread_local! {
    static BRIGHTNESS_MODEL: RefCell<Rc<slint::VecModel<crate::BrightnessItem>>> =
        RefCell::new(Rc::new(slint::VecModel::default()));
}

fn update_slint_ui(
    ui_weak: &slint::Weak<barWindow>,
    monitor_data: &HashMap<DisplayId, (String, u16)>,
) {
    use crate::{BrightnessItem, ExternalBrightnessState};

    let items_to_pass: Vec<(String, String, i32)> = monitor_data
        .iter()
        .map(|(id, (name, brightness))| (id.clone(), name.clone(), *brightness as i32))
        .collect();

    let ui_weak = ui_weak.clone();

    slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let model = crate::helpers::slint_vector::vector::update_vec_model(
                &BRIGHTNESS_MODEL,
                &items_to_pass,
                |(id, name, brightness)| BrightnessItem {
                    id: slint::SharedString::from(id),
                    name: slint::SharedString::from(name),
                    brightness: *brightness,
                },
            );

            let state = ExternalBrightnessState {
                brightness_items: slint::ModelRc::from(model),
            };

            ui.set_extBrightnessData(state);
        }
    })
    .unwrap();
}

pub async fn start_external_brightness(ui_weak: slint::Weak<barWindow>) {
    info!("starting external brightness adjuster");

    let (tx, rx) = tokio::sync::mpsc::channel::<EventToSub>(100);

    if let Some(ui) = ui_weak.upgrade() {
        let tx_refresh = tx.clone();
        ui.on_refresh_ext_brightness(move || {
            let tx = tx_refresh.clone();
            tokio::spawn(async move {
                let _ = tx.send(EventToSub::Refresh).await;
            });
        });

        let tx_set = tx.clone();
        ui.on_set_ext_brightness(move |id, brightness| {
            let target_brightness = brightness.clamp(0, 100) as u16;
            let tx = tx_set.clone();
            let id_string = id.to_string();

            tokio::spawn(async move {
                let _ = tx.send(EventToSub::Set(id_string, target_brightness)).await;
            });
        });
    }

    let worker_ui_weak = ui_weak.clone();
    tokio::spawn(async move {
        run_external_brightness_worker(worker_ui_weak, rx).await;
    });
}

async fn run_external_brightness_worker(
    ui_weak: slint::Weak<barWindow>,
    mut rx: tokio::sync::mpsc::Receiver<EventToSub>,
) {
    let mut failed_attempts = 0;
    let mut duration = Duration::from_millis(50);

    let mut displays: HashMap<DisplayId, Arc<Mutex<Display>>> = HashMap::new();
    let mut monitor_data: HashMap<DisplayId, (String, u16)> = HashMap::new();

    loop {
        tokio::time::sleep(duration).await;
        duration *= 2;

        debug!("start external enumerate");
        let mut some_failed = false;
        displays.clear();
        monitor_data.clear();

        for mut display in Display::enumerate() {
            match display.handle.get_vcp_feature(BRIGHTNESS_CODE) {
                Ok(v) => {
                    let brightness = v.value();
                    let id = display.info.id.clone();
                    let name = display.info.model_name.clone().unwrap_or_default();

                    monitor_data.insert(id.clone(), (name, brightness));
                    displays.insert(id, Arc::new(Mutex::new(display)));
                }
                Err(e) => {
                    warn!("can't get_vcp_feature: {e}");
                    some_failed = true;
                }
            }
        }

        if some_failed {
            failed_attempts += 1;
        }

        if !some_failed || failed_attempts >= 5 {
            break;
        }
    }
    debug!("end external enumerate");

    update_slint_ui(&ui_weak, &monitor_data);

    while let Some(event) = rx.recv().await {
        match event {
            EventToSub::Refresh => {
                for (id, display) in &displays {
                    match display
                        .lock()
                        .unwrap()
                        .handle
                        .get_vcp_feature(BRIGHTNESS_CODE)
                    {
                        Ok(value) => {
                            if let Some(data) = monitor_data.get_mut(id) {
                                data.1 = value.value();
                            }
                        }
                        Err(err) => error!("Refresh failed for monitor {id}: {:?}", err),
                    }
                }
                update_slint_ui(&ui_weak, &monitor_data);
            }

            EventToSub::Set(id, value) => {
                if let Some(display) = displays.get(&id) {
                    let display = Arc::clone(display);

                    let handle = tokio::task::spawn_blocking(move || {
                        if let Err(err) = display
                            .lock()
                            .unwrap()
                            .handle
                            .set_vcp_feature(BRIGHTNESS_CODE, value)
                        {
                            error!("Failed setting DDC brightness payload: {:?}", err);
                        }
                    });

                    let _ = handle.await;

                    if let Some(data) = monitor_data.get_mut(&id) {
                        data.1 = value;
                    }
                    update_slint_ui(&ui_weak, &monitor_data);

                }
            }
        }
    }
}
