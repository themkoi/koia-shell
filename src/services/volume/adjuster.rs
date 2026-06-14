use crate::barWindow;
use std::process::Command;

pub fn start_volume_adjuster(ui_weak: slint::Weak<barWindow>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_set_volume(move |volume, delta| {
            let volume_calc = volume + delta;

            let _ = Command::new("pactl")
                .args(["set-sink-volume", "@DEFAULT_SINK@", &format!("{}%",volume_calc)])
                .spawn();
        });
    }
}
