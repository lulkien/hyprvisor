pub fn get_socket_path() -> String {
    std::env::var("XDG_RUNTIME_DIR")
        .map(|value| format!("{}/hyprvisor2.sock", value))
        .unwrap_or_else(|_| "/tmp/hyprvisor2.sock".to_string())
}
