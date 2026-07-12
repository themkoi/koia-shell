use std::sync::Arc;
use std::time::Duration;
use futures::StreamExt;
use log::info;
use wayle_sysinfo::{SysinfoService};

use crate::barWindow;

pub async fn listen_sysinfo_changes(
    ui_weak: slint::Weak<barWindow>,
    interval: Duration,
) {
    info!("starting unified sysinfo listener");

    let sysinfo_service = Arc::new(SysinfoService::builder().build());
    sysinfo_service.set_cpu_interval(interval);
    sysinfo_service.set_memory_interval(interval);

    let service_clone = sysinfo_service.clone();

    tokio::spawn(async move {
        let ui_init = ui_weak.clone();
        
        let cpu_init = service_clone.cpu.get();
        let mem_init = service_clone.memory.get();

        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_init.upgrade() {
                ui.set_cpuUsage(cpu_init.usage_percent as i32);
                ui.set_cpuTemp(cpu_init.temperature_celsius.unwrap_or(0.0) as i32);
                ui.set_memoryUsage(mem_init.usage_percent as i32);
            }
        });

        let mut cpu_stream = service_clone.cpu.watch();
        let mut mem_stream = service_clone.memory.watch();

        loop {
            tokio::select! {
                Some(cpu) = cpu_stream.next() => {
                    let ui_update = ui_weak.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_update.upgrade() {
                            ui.set_cpuUsage(cpu.usage_percent as i32);
                            ui.set_cpuTemp(cpu.temperature_celsius.unwrap_or(0.0) as i32);
                        }
                    });
                }
                Some(mem) = mem_stream.next() => {
                    let ui_update = ui_weak.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_update.upgrade() {
                            ui.set_memoryUsage(mem.usage_percent as i32);
                        }
                    });
                }
                else => break,
            }
        }
    });
}