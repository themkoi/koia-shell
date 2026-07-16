use futures::StreamExt;
use log::info;
use slint::Model;
use slint::{ModelRc, ToSharedString, VecModel};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use wayle_bluetooth::BluetoothService;

use crate::BluetoothItem;
use crate::barWindow;

thread_local! {
    static CONNECTED_DEVICES_MODEL: RefCell<Rc<VecModel<BluetoothItem>>> = RefCell::new(Rc::new(VecModel::default()));
    static DISCOVERED_DEVICES_MODEL: RefCell<Rc<VecModel<BluetoothItem>>> = RefCell::new(Rc::new(VecModel::default()));
}

fn refresh_bluetooth_ui(ui: &barWindow, bt_service: &Arc<BluetoothService>) {
    let current_connected = ui.get_bluetooth_connected_devices();
    let current_discovered = ui.get_bluetooth_discovered_devices();

    tl_models_init(ui, &current_connected, &current_discovered);

    let available = bt_service.available.get();
    let enabled = bt_service.enabled.get();
    
    ui.set_bluetooth_available(available);
    ui.set_bluetooth_enabled(enabled);

    let mut connected_items = Vec::new();
    let mut discovered_items = Vec::new();

    if available {
        let connected_addresses = bt_service.connected.get(); 
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

    crate::helpers::slint_vector::vector::update_vec_model(
        &CONNECTED_DEVICES_MODEL,
        &connected_items,
        |item| item.clone(),
    );

    crate::helpers::slint_vector::vector::update_vec_model(
        &DISCOVERED_DEVICES_MODEL,
        &discovered_items,
        |item| item.clone(),
    );
}

fn tl_models_init(
    ui: &barWindow, 
    curr_conn: &ModelRc<BluetoothItem>, 
    curr_disc: &ModelRc<BluetoothItem>
) -> (Rc<VecModel<BluetoothItem>>, Rc<VecModel<BluetoothItem>>) {
    CONNECTED_DEVICES_MODEL.with(|conn_cell| {
        DISCOVERED_DEVICES_MODEL.with(|disc_cell| {
            let conn_rc = conn_cell.borrow().clone();
            let disc_rc = disc_cell.borrow().clone();

            if curr_conn.clone().as_any().downcast_ref::<VecModel<BluetoothItem>>().is_none() {
                ui.set_bluetooth_connected_devices(ModelRc::from(conn_rc.clone()));
            }
            if curr_disc.clone().as_any().downcast_ref::<VecModel<BluetoothItem>>().is_none() {
                ui.set_bluetooth_discovered_devices(ModelRc::from(disc_rc.clone()));
            }

            (conn_rc, disc_rc)
        })
    })
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