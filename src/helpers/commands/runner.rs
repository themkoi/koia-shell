use std::thread;
use log::{debug, error};

use crate::AppWindow;

use std::process::Command;

pub fn start_command_handler(ui_weak: slint::Weak<AppWindow>) {
    if let Some(ui) = ui_weak.upgrade() {
        ui.on_run_command(move |command_str| {
            let cmd_to_run = command_str.to_string();

            thread::spawn(move || {
                let output = Command::new("sh").arg("-c").arg(&cmd_to_run).output();

                match output {
                    Ok(out) => {
                        if out.status.success() {
                            let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
                            debug!("Success:\n{}", stdout);

                        } else {
                            let stderr = String::from_utf8_lossy(&out.stderr);
                            error!("Error:\n{}", stderr);
                        }
                    }
                    Err(e) => eprintln!("Failed to start command: {}", e),
                }
            });
        })
    }
}
