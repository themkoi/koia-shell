use log::{error, info};
use wayle_network::NetworkService;

use crate::barWindow;
use std::sync::Arc;

pub async fn start_network_actions(
    ui_weak: slint::Weak<barWindow>,
    network_service: Arc<NetworkService>,
) {
    info!("starting network interaction handler");

    slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let net_toggle = Arc::clone(&network_service);
            ui.on_set_wifi_enabled(move |enabled| {
                let net = Arc::clone(&net_toggle);
                info!("UI requested Wi-Fi change state to: {}", enabled);

                tokio::spawn(async move {
                    if let Some(wifi) = net.wifi.get() {
                        if let Err(e) = wifi.set_enabled(enabled).await {
                            error!("Failed to toggle Wi-Fi radio: {:?}", e);
                        } else {
                            info!("Wi-Fi hardware radio toggled successfully");
                        }
                    }
                });
            });

            let net_disconnect = Arc::clone(&network_service);
            ui.on_disconnect_wifi(move || {
                let net = Arc::clone(&net_disconnect);
                info!("UI requested network disconnection");

                tokio::spawn(async move {
                    if let Some(wifi) = net.wifi.get() {
                        if let Err(e) = wifi.disconnect().await {
                            error!("Failed to disconnect active network: {:?}", e);
                        } else {
                            info!("Successfully disconnected from network");
                        }
                    }
                });
            });

            let net_connect = Arc::clone(&network_service);
            ui.on_connect_wifi(move |target_ssid| {
                let net = Arc::clone(&net_connect);
                let ssid = target_ssid.to_string();

                tokio::spawn(async move {
                    if let Some(wifi) = net.wifi.get() {
                        if let Some(target_ap) = wifi
                            .access_points
                            .get()
                            .iter()
                            .find(|ap| ap.ssid.get() == ssid.clone().into())
                        {
                            log::info!("Connecting to SSID: {}", ssid);

                            let path = target_ap.object_path().clone();
                            if let Err(e) = wifi.connect(path.into(), None).await {
                                error!("Failed to connect to access point: {:?}", e);
                            }
                        }
                    }
                });
            });

            let net_refresh = Arc::clone(&network_service);
            ui.on_refresh_networks(move || {
                let net = Arc::clone(&net_refresh);

                tokio::spawn(async move { net.wifi.get().unwrap().device.request_scan().await });
            });
        }
    })
    .unwrap();
}
