use crate::{
    barWindow, services::network::{actions::start_network_actions, listener::listen_network_changes},

};
use wayle_network::NetworkService;

mod actions;
mod listener;

pub async fn start_network_management(ui_weak: slint::Weak<barWindow>) {
    tokio::spawn(async move {
        let service: std::sync::Arc<NetworkService> = NetworkService::new().await.unwrap().into();

        listen_network_changes(ui_weak.clone(), service.clone()).await;
        start_network_actions(ui_weak, service).await;
    });
}
