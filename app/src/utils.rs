use std::env;

pub fn get_socket_path() -> String {
    match env::var("HYPRLAND_INSTANCE_SIGNATURE") {
        Ok(var) => var,
        Err(_) => panic!("Is hyprland running?"),
    };

    env::var("XDG_RUNTIME_DIR")
        .map(|value| format!("{value}/hyprvisor.sock"))
        .unwrap_or_else(|_| "/tmp/hyprvisor.sock".to_string())
}
