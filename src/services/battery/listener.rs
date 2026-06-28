use futures::stream::{select_all, StreamExt};
use log::info;
use slint::{ToSharedString, VecModel};
use std::rc::Rc;
use upower_dbus::{BatteryState, BatteryType, DeviceProxy, UPowerProxy};

use crate::{barWindow, BatteryDataSlint};

fn determine_icon(kind: BatteryType) -> String {
    match kind {
        BatteryType::Mouse => "mouse icon".to_string(),
        BatteryType::Keyboard => "keyboard icon".to_string(),
        BatteryType::Headphones | BatteryType::Headset => "headphone icon".to_string(),
        _ => "".to_string(),
    }
}

fn determine_name(kind: BatteryType, path_str: &str) -> String {
    let leaf = path_str.split('/').last().unwrap_or("Device");
    format!("{:?}: {}", kind, leaf)
}

struct ManagedDevice {
    path_str: String,
    name: String,
    icon_type: String,
    proxy: DeviceProxy<'static>,
}

pub async fn listen_battery_changes(ui_weak: slint::Weak<barWindow>) {
    info!("starting dynamic upower_dbus battery listener");

    tokio::spawn(async move {
        let connection = zbus::Connection::system().await.unwrap();

        let upower_proxy = UPowerProxy::builder(&connection).build().await.unwrap();

        let mut device_added_stream = upower_proxy.receive_device_added().await.unwrap();
        let mut device_removed_stream = upower_proxy.receive_device_removed().await.unwrap();

        let display_device = upower_proxy.get_display_device().await.unwrap();

        let initial_paths = upower_proxy.enumerate_devices().await.unwrap();
        let mut extra_devices: Vec<ManagedDevice> = Vec::new();

        for path in initial_paths {
            let path_str = path.as_str().to_string();

            if let Ok(device_proxy) = DeviceProxy::builder(&connection)
                .destination("org.freedesktop.UPower")
                .unwrap()
                .path(path)
                .unwrap()
                .build()
                .await
            {
                let kind = device_proxy.type_().await.unwrap_or(BatteryType::Unknown);

                if kind != BatteryType::LinePower && kind != BatteryType::Unknown {
                    extra_devices.push(ManagedDevice {
                        name: determine_name(kind, &path_str),
                        icon_type: determine_icon(kind),
                        path_str,
                        proxy: device_proxy,
                    });
                }
            }
        }

        let mut rebuild_hardware_stream = true;

        let mut hardware_change_stream = select_all(vec![display_device
            .receive_percentage_changed()
            .await
            .map(|_| ())
            .boxed()]);

        loop {
            if rebuild_hardware_stream {
                let mut streams = vec![
                    display_device
                        .receive_percentage_changed()
                        .await
                        .map(|_| ())
                        .boxed(),
                    display_device
                        .receive_state_changed()
                        .await
                        .map(|_| ())
                        .boxed(),
                ];

                for dev in &extra_devices {
                    streams.push(
                        dev.proxy
                            .receive_percentage_changed()
                            .await
                            .map(|_| ())
                            .boxed(),
                    );

                    streams.push(dev.proxy.receive_state_changed().await.map(|_| ()).boxed());
                }

                hardware_change_stream = select_all(streams);
                rebuild_hardware_stream = false;
            }

            let ui_update = ui_weak.clone();

            let current_percentage = display_device.percentage().await.unwrap_or(0.0) as i32;

            let current_state = format!(
                "{:?}",
                display_device
                    .state()
                    .await
                    .unwrap_or(BatteryState::Unknown)
            );

            let mut secondary_snapshots = Vec::new();

            for dev in &extra_devices {
                let pct = dev.proxy.percentage().await.unwrap_or(0.0) as i32;

                let st = format!(
                    "{:?}",
                    dev.proxy.state().await.unwrap_or(BatteryState::Unknown)
                );

                secondary_snapshots.push((dev.name.clone(), st, pct, dev.icon_type.clone()));
            }

            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_update.upgrade() {
                    let mut battery_list = Vec::new();

                    battery_list.push(BatteryDataSlint {
                        name: "Laptop Battery".to_shared_string(),
                        state: current_state.to_shared_string(),
                        percentage: current_percentage,
                        icon_type: "".to_shared_string(),
                    });

                    for (name, state, percentage, icon) in secondary_snapshots {
                        battery_list.push(BatteryDataSlint {
                            name: name.to_shared_string(),
                            state: state.to_shared_string(),
                            percentage,
                            icon_type: icon.to_shared_string(),
                        });
                    }

                    let model = Rc::new(VecModel::from(battery_list));
                    ui.set_batteryData(model.into());
                }
            });

            tokio::select! {
                _ = hardware_change_stream.next() => {}

                Some(signal) = device_added_stream.next() => {
                    if let Ok(args) = signal.args() {
                        let path = args.device;
                        let path_str = path.as_str().to_string();

                        if let Ok(device_proxy) = DeviceProxy::builder(&connection)
                            .destination("org.freedesktop.UPower")
                            .unwrap()
                            .path(path_str.clone())
                            .unwrap()
                            .build()
                            .await
                        {
                            let kind = device_proxy.type_().await.unwrap_or(BatteryType::Unknown);

                            if kind != BatteryType::LinePower && kind != BatteryType::Unknown {
                                info!("New connected upower peripheral detected: {:?}", kind);

                                extra_devices.push(ManagedDevice {
                                    name: determine_name(kind, &path_str),
                                    icon_type: determine_icon(kind),
                                    path_str,
                                    proxy: device_proxy,
                                });

                                rebuild_hardware_stream = true;
                            }
                        }
                    }
                }

                Some(signal) = device_removed_stream.next() => {
                    if let Ok(args) = signal.args() {
                        let path_str = args.device.as_str();
                        let old_len = extra_devices.len();

                        extra_devices.retain(|dev| dev.path_str != path_str);

                        if extra_devices.len() != old_len {
                            info!("Upower peripheral disconnected");
                            rebuild_hardware_stream = true;
                        }
                    }
                }
            }
        }
    });
}
