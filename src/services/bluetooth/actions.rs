use log::{error, info};
use std::sync::Arc;
use std::time::Duration;
use wayle_bluetooth::BluetoothService;

use crate::barWindow;

pub async fn start_bluetooth_actions(
    ui_weak: slint::Weak<barWindow>,
    bt_service: Arc<BluetoothService>,
) {
    info!("starting bluetooth interaction handler");

    slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            
            let bt_toggle = Arc::clone(&bt_service);
            ui.on_set_bluetooth_enabled(move |enabled| {
                let bt = Arc::clone(&bt_toggle);
                info!("UI requested Bluetooth change state to: {}", enabled);

                tokio::spawn(async move {
                    let result = if enabled {
                        bt.enable().await
                        bt.
                    } else {
                        bt.disable().await
                    };

                    if let Err(e) = result {
                        error!("Failed to toggle Bluetooth radio: {:?}", e);
                    } else {
                        info!("Bluetooth hardware radio toggled successfully");
                    }
                });
            });

            let bt_connect = Arc::clone(&bt_service);
            ui.on_connect_bluetooth_device(move |target_address| {
                let bt = Arc::clone(&bt_connect);
                let address = target_address.to_string();

                tokio::spawn(async move {
                    if let Some(device) = bt
                        .devices
                        .get()
                        .iter()
                        .find(|dev| dev.address.get() == address)
                    {
                        info!("Connecting to Bluetooth device: {}", address);
                        if let Err(e) = device.connect().await {
                            error!("Failed to connect to device {}: {:?}", address, e);
                        } else {
                            info!("Successfully connected to device: {}", address);
                        }
                    } else {
                        error!("Could not connect: device {} not found", address);
                    }
                });
            });

            let bt_disconnect = Arc::clone(&bt_service);
            ui.on_disconnect_bluetooth_device(move |target_address| {
                let bt = Arc::clone(&bt_disconnect);
                let address = target_address.to_string();

                tokio::spawn(async move {
                    if let Some(device) = bt
                        .devices
                        .get()
                        .iter()
                        .find(|dev| dev.address.get() == address)
                    {
                        info!("Disconnecting from Bluetooth device: {}", address);
                        if let Err(e) = device.disconnect().await {
                            error!("Failed to disconnect device {}: {:?}", address, e);
                        } else {
                            info!("Successfully disconnected device: {}", address);
                        }
                    }
                });
            });

            let bt_refresh = Arc::clone(&bt_service);
            ui.on_start_bluetooth_discovery(move || {
                let bt = Arc::clone(&bt_refresh);

                tokio::spawn(async move {
                    info!("UI requested Bluetooth device discovery scan");
                    if let Err(e) = bt.start_timed_discovery(Duration::from_secs(30)).await {
                        error!("Failed to request Bluetooth discovery scan: {:?}", e);
                    }
                });
            });
        }
    })
    .unwrap();
}