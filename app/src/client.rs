use crate::{
    common_types::{ClientInfo, SubscriptionID},
    opts::{CommandOpts, SubscribeOpts},
    utils,
};
use std::process;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[allow(unused)]
pub async fn start_client(socket: &str, subscription_opts: &SubscribeOpts) {
    let (sub_id, data_format): (SubscriptionID, u32) = match subscription_opts {
        SubscribeOpts::Workspaces { fix_workspace } => (
            SubscriptionID::Workspaces,
            fix_workspace.map_or(0, |fw| {
                log::warn!("Max workspaces = 10");
                fw.min(10)
            }),
        ),
        SubscribeOpts::Window { title_length } => (
            SubscriptionID::Window,
            title_length.map_or(0, |tl| {
                log::warn!("Max title length = 100");
                tl.min(100)
            }),
        ),
    };

    let pid = process::id();
    let client_info = ClientInfo::new(pid, sub_id);
    let subcribe_message = serde_json::to_string(&client_info).unwrap();

    let max_connect_attempts = 5;
    let attempt_delay = 500;

    let mut connection = match utils::try_connect(socket, max_connect_attempts, attempt_delay).await
    {
        Some(stream) => stream,
        None => {
            log::error!("Failed to connect to socket: {}", socket);
            return;
        }
    };

    if let Err(e) = connection.write_all(subcribe_message.as_bytes()).await {
        log::error!("Failed to subscriber to server");
        return;
    }

    loop {
        let mut buffer: [u8; 1024] = [0; 1024];
        let bytes_received = match connection.read(&mut buffer).await {
            Ok(size) if size > 0 => size,
            Ok(_) | Err(_) => {
                log::error!("Error reading from server.");
                return;
            }
        };

        let response_message = String::from_utf8_lossy(&buffer[..bytes_received]).to_string();

        // response_message = reformat_response(
        //     &response_message,
        //     &self.client_info.subscription_id,
        //     &self.extra_data,
        // );

        println!("{response_message}");
    }
}

pub async fn send_server_command(
    socket_path: &str,
    command: &CommandOpts,
    max_attempts: usize,
) -> bool {
    let message = serde_json::to_string(&command).unwrap();
    match utils::write_to_socket(socket_path, &message, max_attempts, 200).await {
        Some(response) => {
            log::info!("Response: {response}");
            true
        }
        None => false,
    }
}
