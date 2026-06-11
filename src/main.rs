use clap::Parser;
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
    #[arg(short, long, default_value = "DP-3")]
    monitor: String,

    /// Theme mode: dark or light (default: dark)
    #[arg(short, long, value_parser = ["dark", "light"], default_value = "dark")]
    theme: String,
}

slint::include_modules!();
// Generating Spell widgets/windows from slint windows.
spell_framework::generate_widgets![barWindow];

mod config_shell;
use config_shell::components::theme;
use config_shell::config;

mod services;
use crate::{config_shell::config::write_config_slint, services::taskbar::taskbar::run_taskbar};

mod helpers;
use crate::helpers::commands::runner::start_command_handler;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let config = config::load_app_config().unwrap();
    let args = Args::parse();

    let bar_conf = WindowConf::builder()
        .width(Dimension::Full)
        .height(40_u32)
        .anchor_1(LayerAnchor::TOP)
        .margins(5, 0, 0, 10)
        .exclusive_zone(40)
        .layer_type(LayerType::Top)
        .monitor(args.monitor)
        .build()
        .unwrap();

    // Initialising Slint Window and corresponding wayland part.
    let ui = barWindowSpell::invoke_spell("bar", bar_conf);

    if args.theme == "dark" {
        Palette::get(&ui).set_color_scheme(ColorScheme::Dark);

    }
    theme::apply_config_palette(&ui, &config);
    write_config_slint(&config, ui.as_weak());

    run_taskbar(&config, ui.as_weak());
    start_command_handler(ui.as_weak());

    // Calling the event loop function for running the window
    cast_spell!(ui)
}
