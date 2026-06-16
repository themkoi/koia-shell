use log::info;
use crate::barWindow;

pub async fn start_brightness_adjuster(
    config: &crate::config::AppConfig,
    ui_weak: slint::Weak<barWindow>,
) {
    info!("starting brightness adjuster");
    if let Some(ui) = ui_weak.upgrade() {
        let brightness_device = config.config.hardware_config.brightness_device.clone();

        ui.on_set_brightness(move |brightness, delta| {
            let brightness_calc = (brightness + delta).min(100).max(0);
            let device = brightness_device.clone();

            tokio::spawn(async move {
                let max_brightness_path = format!("/sys/class/backlight/{}/max_brightness", device);
                let brightness_path = format!("/sys/class/backlight/{}/brightness", device);

                if let Ok(max_brightness_str) = tokio::fs::read_to_string(&max_brightness_path).await {
                    if let Ok(max_brightness) = max_brightness_str.trim().parse::<u32>() {
                        let actual_brightness =
                            (brightness_calc as f32 / 100.0 * max_brightness as f32) as u32;
                        let _ = tokio::fs::write(&brightness_path, actual_brightness.to_string()).await;
                    }
                }
            });
        });
    }
}