use clap::Parser;
use slint::{ComponentHandle,language::ColorScheme};
use spell_framework::{
    self, cast_spell,
    layer_properties::{Dimension, LayerAnchor, LayerType, WindowConf},
};
use std::{env};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Monitor name (default: focused)
    #[arg(short, long, default_value = "")]
    monitor: String,
}

slint::include_modules!();
// Generating Spell widgets/windows from slint windows.
spell_framework::generate_widgets![barWindow];

mod config_shell;
use config_shell::config;

mod services;
use crate::{
    config_shell::{components::theme::build_config_palette, config::build_config_slint},
    helpers::{
        displays::{display::get_display_info, watcher::start_watcher},
        noctalia,
        touch_area::manager::start_touch_manager,
    },
    noctalia::{is_dark_mode, read_config_summary},
    services::{
        hardware_specific::harware_specific_management,
        power_profiles::start_power_profile_management, taskbar::taskbar::run_taskbar,
        tray::manager::start_system_tray,
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
    let args = Args::parse();

    let monitor: String;

    if args.monitor.is_empty() {
        if let Some((connector_name, _size)) =
            get_display_info(&config.config.default_display, &config.config.default_bar)
        {
            monitor = connector_name;
        } else {
            let (connector_name, _size) =
                get_display_info(&config.config.fallback_display, &config.config.default_bar)
                    .unwrap();
            monitor = connector_name;
        }
    } else {
        monitor = args.monitor;
    }
    let noctalia_status = read_config_summary(Some(config.config.default_bar.as_str())).unwrap();
    let bar_height = noctalia_status.bar_thickness + noctalia_status.bar_margin_edge;
let (connector_name, _size) = get_display_info(
    &config.config.default_display,
    &config.config.default_bar,
).unwrap_or((String::new(), (0, 0)));

    start_watcher(connector_name, config.config.default_bar.clone());

    let bar_conf = WindowConf::builder()
        .width(Dimension::Full)
        .height(bar_height as u32)
        .anchor_1(LayerAnchor::TOP)
        .margins(-(bar_height as i32), 0, 0, 0)
        // .exclusive_zone(config.config.window_config.bar_height.into())
        .layer_type(LayerType::Top)
        .monitor(monitor.clone())
        .build()
        .unwrap();

    let schemes = build_config_palette(&config);

    // bar init
    let bar_ui = barWindowSpell::invoke_spell("bar", bar_conf);
    let window_width_bar = bar_ui.get_window_width();
    let window_height_bar = bar_ui.get_window_height();
    let config_slint = build_config_slint(&config, window_width_bar, bar_height as f32);

    bar_ui.subtract_input_region(0, 0, window_width_bar as i32, window_height_bar as i32);

    if is_dark_mode().unwrap() {
        Palette::get(&bar_ui.ui).set_color_scheme(ColorScheme::Dark);
    }

    MaterialPalette::get(&bar_ui.ui).set_schemes(schemes.clone());
    bar_ui.set_config(config_slint.clone());

    run_taskbar(&config, bar_ui.as_weak()).await;

    harware_specific_management(&config, bar_ui.as_weak()).await;
    start_power_profile_management(bar_ui.as_weak()).await;

    start_touch_manager(&config, &bar_ui);
    start_command_handler(bar_ui.as_weak());

    start_system_tray(&config, bar_ui.as_weak()).await;
    bar_ui.invoke_init_ui();

    // Calling the event loop function for running the window
    cast_spell!(
        windows: [bar_ui],
    )
}
