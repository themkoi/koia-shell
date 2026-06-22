pub fn is_display_connected(target_display: &str) -> bool {
    niri_ipc::socket::Socket::connect()
        .and_then(|mut socket| socket.send(niri_ipc::Request::Outputs))
        .ok()                       
        .and_then(|reply| reply.ok()) 
        .map(|response| match response {
            niri_ipc::Response::Outputs(outputs) => {
                outputs.iter().any(|o| o.0 == target_display)
            }
            _ => false,
        })
        .unwrap_or(false)
}