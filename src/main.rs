use clap::{builder::Str, Parser};
use log::error;
use slint::{language::ColorScheme, ComponentHandle};
use spell_framework::{
    self, cast_spell,
    layer_properties::{Dimension, LayerAnchor, LayerType, WindowConf},
};
use std::{env, error::Error};

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
spell_framework::generate_widgets![barWindow, clipboardWindow];

mod config_shell;
use config_shell::config;

mod services;
use crate::{
    config_shell::{components::theme::build_config_palette, config::build_config_slint},
    services::taskbar::taskbar::run_taskbar,
};

mod helpers;
use crate::helpers::commands::runner::start_command_handler;

fn main() -> Result<(), Box<dyn Error>> {
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
        .height(config.config.total_bar_height as u32)
        .anchor_1(LayerAnchor::TOP)
        .margins(0, 0, 0, 0)
        .exclusive_zone(config.config.bar_height.into())
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

    let schemes = build_config_palette(&config);
    let config_slint = build_config_slint(&config);
    // bar init
    let bar_ui = barWindowSpell::invoke_spell("bar", bar_conf);
    let window_width = bar_ui.get_window_width();
    bar_ui.subtract_input_region(
        0,
        config.config.bar_height.into(),
        window_width as i32,
        config.config.total_bar_height as i32 - config.config.bar_height as i32,
    );

    if args.theme == "dark" {
        Palette::get(&bar_ui.ui).set_color_scheme(ColorScheme::Dark);
    }
    MaterialPalette::get(&bar_ui.ui).set_schemes(schemes.clone());
    bar_ui.set_config(config_slint.clone());

    run_taskbar(&config, bar_ui.as_weak());
    start_command_handler(bar_ui.as_weak());

    // clipboard init
    let clipboard_ui = clipboardWindowSpell::invoke_spell("clipboardWindow", clipboard_conf);

    if args.theme == "dark" {
        Palette::get(&clipboard_ui.ui).set_color_scheme(ColorScheme::Dark);
    }
    MaterialPalette::get(&clipboard_ui.ui).set_schemes(schemes.clone());
    clipboard_ui.set_config(config_slint.clone());
    clipboard_ui.hide();

    // Calling the event loop function for running the window
    cast_spell!(windows: [clipboard_ui,bar_ui])
}
