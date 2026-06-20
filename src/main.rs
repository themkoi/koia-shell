use clap::Parser;
use slint::{language::ColorScheme, ComponentHandle};
use spell_framework::{
    self, cast_spell,
    layer_properties::{Dimension, LayerAnchor, LayerType, WindowConf},
};
use std::env;

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
use config_shell::config;

mod services;
use crate::{
    config_shell::{components::theme::build_config_palette, config::build_config_slint},
    helpers::touch_area::manager::start_touch_manager,
    services::{
        battery::listener::listen_battery_changes,
        brightness::start_brightness_management,
        notifications::manager::{start_notification_service},
        power_profiles::start_power_profile_management,
        taskbar::taskbar::run_taskbar,
        time::provider::provide_time,
        tray::manager::start_system_tray,
        volume::start_volume_management,
    },
};

mod helpers;
use crate::helpers::commands::runner::start_command_handler;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let config = config::load_app_config().unwrap();
    let args = Args::parse();

    let monitor: String;

    if args.monitor.is_empty() {
        monitor = config.config.default_display.clone();
    } else {
        monitor = args.monitor;
    }

    let bar_conf = WindowConf::builder()
        .width(Dimension::Full)
        .height(Dimension::Full)
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
        .height(
            config
                .config
                .window_config
                .notification_window_height
                .clone() as u32,
        )
        .monitor(config.config.window_config.notification_screen.clone())
        .anchor_1(LayerAnchor::TOP)
        .anchor_2(LayerAnchor::RIGHT)
        .margins(0, 0, 0, 0)
        .layer_type(LayerType::Top)
        .build()
        .unwrap();

    let schemes = build_config_palette(&config);
    let config_slint = build_config_slint(&config);
    // bar init
    let bar_ui = barWindowSpell::invoke_spell("bar", bar_conf);
    let window_width = bar_ui.get_window_width();
    let window_height = bar_ui.get_window_height();

    bar_ui.subtract_input_region(
        0,
        config.config.window_config.bar_height.into(),
        window_width as i32,
        (window_height - config.config.window_config.bar_height as f32) as i32,
    );

    if args.theme == "dark" {
        Palette::get(&bar_ui.ui).set_color_scheme(ColorScheme::Dark);
    }
    MaterialPalette::get(&bar_ui.ui).set_schemes(schemes.clone());
    bar_ui.set_config(config_slint.clone());

    run_taskbar(&config, bar_ui.as_weak()).await;

    start_volume_management(bar_ui.as_weak()).await;
    start_brightness_management(&config, bar_ui.as_weak()).await;
    start_power_profile_management(bar_ui.as_weak()).await;
    listen_battery_changes(bar_ui.as_weak()).await;
    provide_time(bar_ui.as_weak()).await;

    start_touch_manager(&config, window_width, window_height, &bar_ui);
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

    start_notification_service(&notification_ui, &notification_ui.ui);

    notification_ui.subtract_input_region(
        0,
        0,
        config
            .config
            .window_config
            .notification_window_width
            .clone() as i32,
        config
            .config
            .window_config
            .notification_window_height
            .clone() as i32,
    );

    // Calling the event loop function for running the window
    cast_spell!(
        windows: [clipboard_ui, bar_ui],
        notification: notification_ui
    )
}
