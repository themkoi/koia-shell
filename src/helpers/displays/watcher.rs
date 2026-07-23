use std::{env, error::Error, process::{self, Command}, thread, time::Duration};

use smithay_client_toolkit::{
    delegate_output, delegate_registry,
    output::{OutputHandler, OutputState},
    reexports::client::{
        Connection, QueueHandle, globals::registry_queue_init, protocol::wl_output,
    },
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
};

fn restart_and_exit() {
    match env::current_exe() {
        Ok(exe_path) => {
            if let Err(err) = Command::new(exe_path)
                .args(env::args().skip(1)) // Preserves command-line arguments
                .spawn()
            {
                log::error!("Failed to restart process: {}", err);
            }
        }
        Err(err) => {
            log::error!("Failed to get executable path: {}", err);
        }
    }
    process::exit(0);
}

use crate::helpers::displays::display::get_display_info;

pub fn start_watcher(target: String, target_bar: String) {
    std::thread::spawn(move || {
        if let Err(err) = run(target, target_bar) {
            log::error!("Display watcher stopped: {}", err);
        }
    });
}

fn run(target: String, target_bar: String) -> Result<(), Box<dyn Error>> {
    let conn = Connection::connect_to_env()?;

    let (globals, mut event_queue) = registry_queue_init(&conn)?;
    let qh = event_queue.handle();

    let registry_state = RegistryState::new(&globals);
    let output_state = OutputState::new(&globals, &qh);

    let mut watcher = OutputWatcher {
        registry_state,
        output_state,
        initialized: false,
        target,
        target_bar,
    };

    // Initial output discovery
    event_queue.blocking_dispatch(&mut watcher)?;
    watcher.initialized = true;

    loop {
        event_queue.blocking_dispatch(&mut watcher)?;
    }
}

struct OutputWatcher {
    registry_state: RegistryState,
    output_state: OutputState,

    initialized: bool,

    target: String,
    target_bar: String,
}

impl AsMut<OutputState> for OutputWatcher {
    fn as_mut(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
}

impl OutputWatcher {
    fn refresh(&self) {
        match get_display_info(&self.target, &self.target_bar) {
            Some((connector, size)) => {
                log::debug!("Display refresh complete: {} {:?}", connector, size);
            }
            None => {
                log::debug!("Target display '{}' not found", self.target);
            }
        }
    }
}

impl OutputHandler for OutputWatcher {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
        if self.initialized {
            let mut target_connected = false;

            if let Some(info) = self.output_state.info(&output) {
                if let Some(name) = info.name {
                    // Triggers if target matches OR if target is empty
                    if self.target.is_empty() || name == self.target {
                        target_connected = true;
                    }
                }
            }

            if target_connected {
                log::info!(
                    "Display change detected (target: '{}'). Restarting process in 5 seconds...",
                    self.target
                );
                
                thread::sleep(Duration::from_secs(5));
                restart_and_exit();
            }

            log::info!("New display detected");
            self.refresh();
        }
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
        if self.initialized {
            let mut target_removed = false;

            if let Some(info) = self.output_state.info(&output) {
                if let Some(name) = info.name {
                    // Triggers if target matches OR if target is empty
                    if self.target.is_empty() || name == self.target {
                        target_removed = true;
                    }
                }
            }

            if target_removed {
                log::info!(
                    "Display removed (target: '{}'). Restarting process in 5 seconds...",
                    self.target
                );

                thread::sleep(Duration::from_secs(5));
                restart_and_exit();
            }

            log::info!("Display removed");
            self.refresh();
        }
    }
}

// Delegating macros for Smithay Client Toolkit
delegate_registry!(OutputWatcher);
delegate_output!(OutputWatcher);

impl ProvidesRegistryState for OutputWatcher {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers! {
        OutputState,
    }
}
