use futures::stream::select;
use futures::StreamExt;
use log::info;
use slint::ToSharedString;
use wayle_battery::BatteryService;

use crate::{barWindow, BatteryDataSlint};

pub async fn listen_battery_changes(ui_weak: slint::Weak<barWindow>) {
    info!("starting battery listener");
    tokio::spawn(async move {
        let service = BatteryService::new().await.unwrap();
        let ui_init = ui_weak.clone();

        let percentage_init = service.device.percentage.get().clone() as i32;
        let state_init = service.device.state.get().to_shared_string().clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_init.upgrade() {
                let battery_data = BatteryDataSlint {
                    percentage: percentage_init,
                    state: state_init,
                };
                ui.set_batteryData(battery_data);
            }
        });

        let state = service.device.state.clone();
        let percentage = service.device.percentage.clone();

        let mut stream = select(state.watch().map(|_| ()), percentage.watch().map(|_| ()));

        while stream.next().await.is_some() {
            let ui_update = ui_weak.clone();

            let current_state = state.get().to_shared_string();
            let current_percentage = percentage.get() as i32;

            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_update.upgrade() {
                    ui.set_batteryData(BatteryDataSlint {
                        state: current_state,
                        percentage: current_percentage,
                    });
                }
            });
        }
    });
}
