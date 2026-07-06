use crate::{
    barWindow, services::bluetooth::{actions::start_bluetooth_actions, listener::listen_bluetooth_changes},

};
use wayle_bluetooth::BluetoothService;

mod actions;
mod listener;

pub async fn start_bluetooth_management(ui_weak: slint::Weak<barWindow>) {
    tokio::spawn(async move {
        let service: std::sync::Arc<BluetoothService> = BluetoothService::new().await.unwrap().into();

        listen_bluetooth_changes(ui_weak.clone(), service.clone()).await;
        start_bluetooth_actions(ui_weak.clone(), service).await;
    });
}
