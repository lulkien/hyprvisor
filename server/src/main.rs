mod common;
mod hypr_listener;
mod server;

use server::Server;
use std::env;
use std::ops::Add;

#[tokio::main]
async fn main() {
    let socket_path: String = match env::var("XDG_RUNTIME_DIR") {
        Ok(value) => value.add("/hyprvisor.sock"),
        Err(_) => "/tmp/hyprvisor.sock".to_string(),
    };

    let mut server = Server::new();
    server.prepare(socket_path).await;
    server.start().await;
}
