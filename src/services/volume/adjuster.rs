use crate::barWindow;
use std::process::Command;

pub fn start_volume_adjuster(ui_weak: slint::Weak<barWindow>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_set_volume(move |volume, delta| {
            let volume_calc = (volume + delta).clamp(0, 100);

            let _ = Command::new("pamixer")
                .args(["--set-volume", &volume_calc.to_string()])
                .spawn();
        });
    }
}
