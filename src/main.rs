use clap::Parser;
use slint::{ComponentHandle, language::ColorScheme};
use spell_framework::{
    self, cast_spell,
    layer_properties::{Dimension, LayerAnchor, LayerType, WindowConf},
};
use std::{env, time::Duration};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Monitor name (default: focused)
    #[arg(short, long, default_value = "")]
    monitor: String,

    /// Theme mode: dark or light (default: dark)
    #[arg(short, long, value_parser = ["dark", "light"], default_value = "dark")]
    theme: String,
}

slint::include_modules!();
// Generating Spell widgets/windows from slint windows.
spell_framework::generate_widgets![barWindow, clipboardWindow, notificationWindow];

mod config_shell;
mod data_shell;
use config_shell::config;

mod services;
use crate::{
    config_shell::{components::theme::build_config_palette, config::build_config_slint}, data_shell::data::{SessionData, build_session_data_slint, load_or_create_session_data}, helpers::{displays::display::get_display_info, touch_area::manager::start_touch_manager}, services::{
        battery::listener::listen_battery_changes, bluetooth::start_bluetooth_management,
        brightness::start_brightness_management, hardware_specific::harware_specific_management,
        network::start_network_management, notifications::manager::start_notification_service,
        power_profiles::start_power_profile_management, taskbar::taskbar::run_taskbar,
        time::provider::provide_time, tray::manager::start_system_tray,
        volume::start_volume_management, sys_info::listener::listen_sysinfo_changes,
    },
};

mod helpers;
use crate::helpers::commands::runner::start_command_handler;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );
    let config = config::load_app_config().unwrap();
    let data = load_or_create_session_data(&config.config).unwrap();
    let args = Args::parse();

    let monitor: String;
    let mut display_size_bar: (u16, u16) = (1080,1920);

    if args.monitor.is_empty() {
        if let Some((connector_name, size)) = get_display_info(&config.config.default_display) {
            monitor = connector_name;
            display_size_bar.1 = size.1 as u16;
        } else {
            let (connector_name, size) = get_display_info(&config.config.fallback_display).unwrap();
            monitor = connector_name;
            display_size_bar.1 = size.1 as u16;
        }
    } else {
        monitor = args.monitor;
    }

    let (connector_notification, size_notification) =
        get_display_info(&config.config.window_config.notification_screen).unwrap();

    let bar_conf = WindowConf::builder()
        .width(Dimension::Full)
        .height(display_size_bar.1 as u32)
        .anchor_1(LayerAnchor::TOP)
        .margins(0, 0, 0, 0)
        .exclusive_zone(config.config.window_config.bar_height.into())
        .layer_type(LayerType::Top)
        .monitor(monitor.clone())
        .build()
        .unwrap();

    let clipboard_conf = WindowConf::builder()
        .width(400_u32)
        .height(500_u32)
        .margins(0, 0, 0, 0)
        .layer_type(LayerType::Top)
        .monitor(monitor.clone())
        .build()
        .unwrap();

    let notification_conf = WindowConf::builder()
        .width(
            config
                .config
                .window_config
                .notification_window_width
                .clone() as u32,
        )
        .height(size_notification.1 as u32)
        .monitor(connector_notification)
        .anchor_1(LayerAnchor::TOP)
        .anchor_2(LayerAnchor::RIGHT)
        .margins(0, 0, 0, 0)
        .layer_type(LayerType::Overlay)
        .build()
        .unwrap();

    let schemes = build_config_palette(&config);
    let session_data_slint = build_session_data_slint(&data);

    // bar init
    let bar_ui = barWindowSpell::invoke_spell("bar", bar_conf);
    let window_width_bar = bar_ui.get_window_width();
    let window_height_bar = bar_ui.get_window_height();
    let config_slint = build_config_slint(&config, window_width_bar);

    bar_ui.subtract_input_region(
        0,
        config.config.window_config.bar_height.into(),
        window_width_bar as i32,
        (window_height_bar as f32 - config.config.window_config.bar_height as f32) as i32,
    );

    if args.theme == "dark" {
        Palette::get(&bar_ui.ui).set_color_scheme(ColorScheme::Dark);
    }
    MaterialPalette::get(&bar_ui.ui).set_schemes(schemes.clone());
    bar_ui.set_config(config_slint.clone());
    bar_ui.set_sessionData(session_data_slint.clone());
    bar_ui.invoke_init_ui();

    let config_clone = config.config.clone();

    bar_ui.on_write_session_data(move |ui_session_data| {

        let session_data: SessionData = ui_session_data.into();
        let config = config_clone.clone();

        tokio::spawn(async move {
            if let Err(e) = crate::data_shell::data::save_session_data(&config, session_data) {
                log::error!("Failed to save session data: {:?}", e);
            }
        });
    });
    run_taskbar(&config, bar_ui.as_weak()).await;

    start_volume_management(
        bar_ui.as_weak(),
        config.config.interaction_config.allow_overflow_volume,
    )
    .await;
    start_brightness_management(&config, bar_ui.as_weak()).await;
    harware_specific_management(&config, bar_ui.as_weak()).await;
    start_power_profile_management(bar_ui.as_weak()).await;
    listen_sysinfo_changes(bar_ui.as_weak(),Duration::from_secs(config.config.hardware_config.sys_info_polling_duration.into())).await;
    listen_battery_changes(bar_ui.as_weak()).await;
    provide_time(bar_ui.as_weak()).await;

    start_network_management(bar_ui.as_weak()).await;
    start_bluetooth_management(bar_ui.as_weak()).await;

    start_touch_manager(&config, window_width_bar, window_height_bar as f32, &bar_ui);
    start_command_handler(bar_ui.as_weak());

    start_system_tray(&config, bar_ui.as_weak()).await;

    // clipboard init
    let clipboard_ui = clipboardWindowSpell::invoke_spell("clipboardWindow", clipboard_conf);

    if args.theme == "dark" {
        Palette::get(&clipboard_ui.ui).set_color_scheme(ColorScheme::Dark);
    }
    MaterialPalette::get(&clipboard_ui.ui).set_schemes(schemes.clone());
    clipboard_ui.set_config(config_slint.clone());
    clipboard_ui.hide();

    // notification init
    let notification_ui =
        notificationWindowSpell::invoke_spell("notificationWindow", notification_conf);

    if args.theme == "dark" {
        Palette::get(&notification_ui.ui).set_color_scheme(ColorScheme::Dark);
    }
    MaterialPalette::get(&notification_ui.ui).set_schemes(schemes.clone());

    notification_ui.set_config(config_slint.clone());

    let window_height_notifi = size_notification.1;

    notification_ui.subtract_input_region(
        0,
        0,
        config
            .config
            .window_config
            .notification_window_width
            .clone() as i32,
        window_height_notifi as i32,
    );
    start_notification_service(config, &notification_ui).await;

    // Calling the event loop function for running the window
    cast_spell!(
        windows: [clipboard_ui, bar_ui],
        notification: notification_ui
    )
}
