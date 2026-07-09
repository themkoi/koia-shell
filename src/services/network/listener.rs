use futures::StreamExt;
use log::info;
use slint::{ModelRc, ToSharedString, VecModel};
use std::rc::Rc;
use std::sync::Arc;
use wayle_network::NetworkService;

use crate::CompleteNetworkState;
use crate::NetworkItem;
use crate::barWindow;

fn refresh_network_ui(ui: &barWindow, network_service: &Arc<NetworkService>) {
    let mut wifi_enabled = false;
    let mut current_ssid = String::new();
    let mut wifi_ip = "No Hardware".to_string();
    let mut wifi_items = Vec::new();
    let mut current_strength: u8 = u8::MAX;

    if let Some(wifi) = network_service.wifi.get() {
        wifi_enabled = wifi.enabled.get();
        current_ssid = wifi.ssid.get().unwrap_or_default();
        current_strength = wifi.strength.get().unwrap_or_default();

        wifi_ip = wifi
            .ip4_address
            .get()
            .unwrap_or_else(|| "No IP Address".to_string());

        wifi_items = wifi
            .access_points
            .get()
            .iter()
            .filter(|ap| {
                let ssid = ap.ssid.get();
                !ssid.is_empty()
            })
            .map(|ap| {
                let name = ap.ssid.get().to_string();

                let sec_status = format!("{:?}", ap.security.get());
                let locked = !sec_status.contains("None") && !sec_status.is_empty();

                NetworkItem {
                    ssid: name.to_shared_string(),
                    strength: ap.strength.get() as i32,
                    locked,
                }
            })
            .filter(|item| item.ssid.as_str() != current_ssid.as_str())
            .collect();
    }

    let (wired_connected, wired_ip) = match network_service.wired.get() {
        Some(wired) => {
            let ip = wired
                .ip4_address
                .get()
                .unwrap_or_else(|| "Disconnected".to_string());

            let status = format!("{:?}", wired.connectivity.get());
            let is_connected = !status.contains("None")
                && !status.contains("Disconnected")
                && ip != "Disconnected";

            (is_connected, ip)
        }
        None => (false, "Disconnected".to_string()),
    };

    ui.set_networkData(CompleteNetworkState {
        wifi_enabled,
        current_ssid: current_ssid.to_shared_string(),
        current_strength: current_strength.into(),
        wifi_networks: ModelRc::from(Rc::new(VecModel::from(wifi_items))),
        wifi_ip: wifi_ip.to_shared_string(),
        wired_connected,
        wired_ip: wired_ip.to_shared_string(),
    });
}

pub async fn listen_network_changes(
    ui_weak: slint::Weak<barWindow>,
    network_service: Arc<NetworkService>,
) {
    info!("starting network listener");

    tokio::spawn(async move {
        let mut primary_stream = network_service.primary.watch();
        let mut wired_device_stream = network_service.wired.watch();
        let mut wifi_device_stream = network_service.wifi.watch();

        loop {
            let ui_update = ui_weak.clone();
            let net_update = network_service.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_update.upgrade() {
                    refresh_network_ui(&ui, &net_update);
                }
            });

            let wired_device = network_service.wired.get();
            let wifi_device = network_service.wifi.get();

            match (wired_device, wifi_device) {
                (Some(wired), Some(wifi)) => {
                    let mut wired_ip_stream = wired.ip4_address.watch();
                    let mut wired_conn_stream = wired.connectivity.watch();
                    let mut wifi_stream = wifi.watch();
                    let mut wifi_toggle_stream = wifi.enabled.watch();

                    loop {
                        tokio::select! {
                            biased;
                            _ = primary_stream.next() => break,
                            _ = wired_device_stream.next() => break,
                            _ = wifi_device_stream.next() => break,

                            _ = wired_ip_stream.next() => {}
                            _ = wired_conn_stream.next() => {}
                            _ = wifi_stream.next() => {}
                            _ = wifi_toggle_stream.next() => {}
                        }

                        let ui_update = ui_weak.clone();
                        let net_update = network_service.clone();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_update.upgrade() {
                                refresh_network_ui(&ui, &net_update);
                            }
                        });
                    }
                }
                (Some(wired), None) => {
                    let mut wired_ip_stream = wired.ip4_address.watch();
                    let mut wired_conn_stream = wired.connectivity.watch();

                    loop {
                        tokio::select! {
                            biased;
                            _ = primary_stream.next() => break,
                            _ = wired_device_stream.next() => break,
                            _ = wifi_device_stream.next() => break,

                            _ = wired_ip_stream.next() => {}
                            _ = wired_conn_stream.next() => {}
                        }

                        let ui_update = ui_weak.clone();
                        let net_update = network_service.clone();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_update.upgrade() {
                                refresh_network_ui(&ui, &net_update);
                            }
                        });
                    }
                }
                (None, Some(wifi)) => {
                    let mut wifi_stream = wifi.watch();
                    let mut wifi_toggle_stream = wifi.enabled.watch();

                    loop {
                        tokio::select! {
                            biased;
                            _ = primary_stream.next() => break,
                            _ = wired_device_stream.next() => break,
                            _ = wifi_device_stream.next() => break,

                            _ = wifi_stream.next() => {}
                            _ = wifi_toggle_stream.next() => {}
                        }

                        let ui_update = ui_weak.clone();
                        let net_update = network_service.clone();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_update.upgrade() {
                                refresh_network_ui(&ui, &net_update);
                            }
                        });
                    }
                }
                (None, None) => {
                    tokio::select! {
                        biased;
                        _ = primary_stream.next() => {}
                        _ = wired_device_stream.next() => {}
                        _ = wifi_device_stream.next() => {}
                    }
                }
            }
        }
    });
}
