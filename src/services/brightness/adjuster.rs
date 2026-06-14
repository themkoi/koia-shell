use crate::barWindow;
use std::fs;

pub fn start_brightness_adjuster(
    config: &crate::config::AppConfig,
    ui_weak: slint::Weak<barWindow>,
) {
    if let Some(ui) = ui_weak.upgrade() {
        let brightness_device = config.config.hardware_config.brightness_device.clone();

        ui.on_set_brightness(move |brightness, delta| {
            let brightness_calc = (brightness + delta).min(100).max(0);

            let max_brightness_path =
                format!("/sys/class/backlight/{}/max_brightness", brightness_device);
            let brightness_path = format!("/sys/class/backlight/{}/brightness", brightness_device);

            if let Ok(max_brightness_str) = fs::read_to_string(&max_brightness_path) {
                if let Ok(max_brightness) = max_brightness_str.trim().parse::<u32>() {
                    // brightness_calc is percentage (0-100), convert to actual value
                    let actual_brightness =
                        (brightness_calc as f32 / 100.0 * max_brightness as f32) as u32;
                    let _ = fs::write(&brightness_path, actual_brightness.to_string());
                }
            }
        });
    }
}
