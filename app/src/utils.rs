use std::env;

pub fn get_socket_path() -> String {
    env::var("XDG_RUNTIME_DIR")
        .map(|value| format!("{}/hyprvisor2.sock", value))
        .unwrap_or_else(|_| "/tmp/hyprvisor2.sock".to_string())
}

#[allow(unused)]
pub fn get_hyprland_event_socket() -> Option<String> {
    match env::var("HYPRLAND_INSTANCE_SIGNATURE") {
        Ok(value) => Some(format!("/tmp/hypr/{}/.socket2.sock", value)),
        Err(_) => {
            log::error!("HYPRLAND_INSTANCE_SIGNATURE not set! (is hyprland running?)");
            None
        }
    }
}

#[allow(unused)]
pub fn get_hyprland_command_socket() -> Option<String> {
    match env::var("HYPRLAND_INSTANCE_SIGNATURE") {
        Ok(value) => Some(format!("/tmp/hypr/{}/.socket.sock", value)),
        Err(_) => {
            log::error!("HYPRLAND_INSTANCE_SIGNATURE not set! (is hyprland running?)");
            None
        }
    }
}
