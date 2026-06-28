use crate::barWindow;
use log::{error, info};
use tokio::process::Command;

pub async fn start_fan_profile_adjuster_framework(
    config: &crate::config::AppConfig,
    ui_weak: slint::Weak<barWindow>,
) {
    if config.config.hardware_config.hardware_specific_features {
        info!("starting fan profile adjuster");

        if let Some(ui) = ui_weak.upgrade() {
            ui.on_set_fan_profile(move |strategy| {
                let strategy = strategy.to_string().to_lowercase();
        info!("setting fan strategy: {}", strategy);

                tokio::spawn(async move {
                    match Command::new("fw-fanctrl-rs")
                        .arg("use")
                        .arg(&strategy)
                        .status()
                        .await
                    {
                        Ok(status) if status.success() => {}
                        Ok(status) => {
                            error!("fw-fanctrl-rs exited with status: {}", status);
                        }
                        Err(e) => {
                            error!("failed to run fw-fanctrl-rs: {:?}", e);
                        }
                    }
                });
            });
        }
    }
}
