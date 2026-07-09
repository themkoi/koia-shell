/// Finds a connected display by its connector name or description substring,
/// returns its connector name and current logical size, and logs an info message if not found.
pub fn get_display_info(target: &str) -> Option<(String, (i32, i32))> {
    let mut socket = niri_ipc::socket::Socket::connect().ok()?;
    let reply = socket.send(niri_ipc::Request::Outputs).ok()?;
    let response = reply.ok()?;

    if let niri_ipc::Response::Outputs(outputs) = response {
        for (connector_name, output_details) in outputs {
            let target_lower = target.to_lowercase();
            
            // 1. Match against connector name (e.g., "DP-3")
            let matches_connector = connector_name.to_lowercase() == target_lower;
            
            // 2. Match against any part of the target string in 'make' or 'model'
            // We strip out common spacing to bypass minor formatting mismatches
            let make_clean = output_details.make.to_lowercase();
            let model_clean = output_details.model.to_lowercase();
            let combined = format!("{} {}", make_clean, model_clean);

            let matches_description = combined.contains(&target_lower) 
                || target_lower.contains(&make_clean) 
                || target_lower.contains(&model_clean);

            if matches_connector || matches_description {
                let size = output_details.current_mode
                    .and_then(|idx| output_details.modes.get(idx))
                    .map(|mode| (mode.width as i32, mode.height as i32))
                    .unwrap_or((0, 0));

                return Some((connector_name, size));
            }
        }
    }

    log::info!("Display '{}' does not exist or is not connected.", target);
    None
}