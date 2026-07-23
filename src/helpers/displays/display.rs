use std::process::Command;

pub fn get_display_info(target: &str, target_bar: &String) -> Option<(String, (i32, i32))> {
    let mut socket = niri_ipc::socket::Socket::connect().ok()?;
    let reply = socket.send(niri_ipc::Request::Outputs).ok()?;
    let response = reply.ok()?;
    if let niri_ipc::Response::Outputs(outputs) = response {
        let target_lower = target.to_lowercase();

        let mut matched_connector = None;
        let mut matched_size = (0, 0);

        for (connector_name, output_details) in &outputs {
            let matches_connector = connector_name.to_lowercase() == target_lower;

            let make_clean = output_details.make.to_lowercase();
            let model_clean = output_details.model.to_lowercase();
            let combined = format!("{} {}", make_clean, model_clean);

            let matches_description = combined.contains(&target_lower)
                || target_lower.contains(&make_clean)
                || target_lower.contains(&model_clean);

            if matches_connector || matches_description {
                matched_connector = Some(connector_name.clone());
                matched_size = output_details
                    .current_mode
                    .and_then(|idx| output_details.modes.get(idx))
                    .map(|mode| (mode.width as i32, mode.height as i32))
                    .unwrap_or((0, 0));
                break;
            }
        }

        let target_connector = matched_connector?;

        for (connector_name, _) in &outputs {
            let action = if *connector_name == target_connector {
                "bar-show"
            } else {
                "bar-hide"
            };

            let _ = Command::new("noctalia")
                .args(["msg", action, &target_bar, connector_name])
                .status();
        }

        return Some((target_connector, matched_size));
    }

    log::info!("Display '{}' does not exist or is not connected.", target);
    None
}
