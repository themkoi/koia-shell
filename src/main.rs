use clap::Parser;
use slint::{language::ColorScheme, ComponentHandle};
use spell_framework::{
    self, cast_spell,
    layer_properties::{LayerAnchor, LayerType, WindowConf},
};
use std::{env, error::Error};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Monitor width (default: 2560)
    #[arg(short = 'w', long, default_value = "2560")]
    monitor_width: u32,

    /// Monitor name (default: focused)
    #[arg(short, long, default_value = "")]
    monitor: String,

    /// Theme mode: dark or light (default: dark)
    #[arg(short, long, value_parser = ["dark", "light"], default_value = "dark")]
    theme: String,
}

slint::include_modules!();
// Generating Spell widgets/windows from slint windows.
spell_framework::generate_widgets![AppWindow];

mod config_shell;
use config_shell::{config, theme};

fn main() -> Result<(), Box<dyn Error>> {
    let config = config::load_app_config().unwrap();
    let args = Args::parse();

    let window_conf = WindowConf::builder()
        .width(args.monitor_width)
        .height(40_u32)
        .anchor_1(LayerAnchor::TOP)
        .margins(5, 0, 0, 10)
        .exclusive_zone(40)
        .layer_type(LayerType::Top)
        .monitor(args.monitor)
        .build()
        .unwrap();

    // Initialising Slint Window and corresponding wayland part.
    let ui = AppWindowSpell::invoke_spell("bar", window_conf);

    if args.theme == "dark" {
        ui.set_color_scheme(ColorScheme::Dark);
    }
    theme::apply_config_palette(&ui, &config);

    // Setting the callback closure value which will be called on when the button is clicked.
    ui.on_request_increase_value({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_counter(ui.get_counter() + 1);
        }
    });

    // Calling the event loop function for running the window
    cast_spell!(ui)
}
