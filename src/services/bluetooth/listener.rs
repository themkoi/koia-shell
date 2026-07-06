use futures::StreamExt;
use log::info;
use slint::{ModelRc, ToSharedString, VecModel};
use std::rc::Rc;
use std::sync::Arc;
use wayle_bluetooth::BluetoothService;

use crate::BluetoothItem;
use crate::CompleteBluetoothState;
use crate::barWindow;

fn refresh_bluetooth_ui(ui: &barWindow, bt_service: &Arc<BluetoothService>) {
    let available = bt_service.available.get();
    let enabled = bt_service.enabled.get();
    
    let connected_addresses = bt_service.connected.get(); 
    
    let mut connected_items = Vec::new();
    let mut discovered_items = Vec::new();

    if available {
        let (connected, discovered): (Vec<_>, Vec<_>) = bt_service
            .devices
            .get()
            .iter()
            .map(|device| {
                let name = device.name.get().unwrap_or_else(|| "Unknown Device".to_string());
                let address = device.address.get();
                let is_connected = connected_addresses.contains(&address);
                
                let dev_type = device.icon.get().unwrap_or_else(|| "unknown".to_string()); 

                BluetoothItem {
                    name: name.to_shared_string(),
                    address: address.to_shared_string(),
                    connected: is_connected,
                    device_type: dev_type.to_shared_string(),
                }
            })
            .partition(|item| item.connected);

        connected_items = connected;
        discovered_items = discovered;
    }

    ui.set_bluetoothData(CompleteBluetoothState {
        available,
        enabled,
        connected_devices: ModelRc::from(Rc::new(VecModel::from(connected_items))),
        discovered_devices: ModelRc::from(Rc::new(VecModel::from(discovered_items))),
    });
}

pub async fn listen_bluetooth_changes(
    ui_weak: slint::Weak<barWindow>,
    bt_service: Arc<BluetoothService>,
) {
    info!("starting bluetooth listener");

    tokio::spawn(async move {
        let mut available_stream = bt_service.available.watch();
        let mut enabled_stream = bt_service.enabled.watch();
        let mut devices_stream = bt_service.devices.watch();
        let mut connected_stream = bt_service.connected.watch();

        loop {
            let ui_update = ui_weak.clone();
            let bt_update = bt_service.clone();
            slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_update.upgrade() {
                    refresh_bluetooth_ui(&ui, &bt_update);
                }
            }).unwrap();

            tokio::select! {
                biased;
                _ = available_stream.next() => {}
                _ = enabled_stream.next() => {}
                _ = devices_stream.next() => {}
                _ = connected_stream.next() => {}
            }
        }
    });
}