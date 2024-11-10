use crate::{
    error::{HyprvisorError, HyprvisorResult},
    hyprland::types::HyprSocketType,
};
use std::{env, time::Duration};
use tokio::net::UnixStream;

pub fn get_socket_path() -> String {
    match env::var("HYPRLAND_INSTANCE_SIGNATURE") {
        Ok(var) => var,
        Err(_) => panic!("Is hyprland running?"),
    };

    env::var("XDG_RUNTIME_DIR")
        .map(|value| format!("{value}/hyprvisor.sock"))
        .unwrap_or_else(|_| "/tmp/hyprvisor.sock".to_string())
}

pub fn get_hyprland_socket(socket_type: &HyprSocketType) -> String {
    let instance_signature = match env::var("HYPRLAND_INSTANCE_SIGNATURE") {
        Ok(var) => var,
        Err(_) => panic!("Is hyprland running?"),
    };

    let socket_name = match socket_type {
        HyprSocketType::Command => ".socket.sock",
        HyprSocketType::Event => ".socket2.sock",
    };

    env::var("XDG_RUNTIME_DIR")
        .map(|value| format!("{value}/hypr/{instance_signature}/{socket_name}"))
        .unwrap_or_else(|_| format!("/tmp/hypr/{instance_signature}/{socket_name}"))
}

pub async fn try_connect(
    socket_path: &str,
    max_try: usize,
    delay: u64,
) -> HyprvisorResult<UnixStream> {
    for attempt in 0..max_try {
        if let Ok(stream) = UnixStream::connect(socket_path).await {
            return Ok(stream);
        }
        log::debug!("Try connect: {} | Attempt: {}", socket_path, attempt + 1);
        tokio::time::sleep(Duration::from_millis(delay)).await;
    }

    log::warn!("Failed to connect to socket: {socket_path}");
    Err(HyprvisorError::StreamError)
}

pub async fn try_write(stream: &UnixStream, content: &str) -> HyprvisorResult<()> {
    if let Err(e) = stream.writable().await {
        log::error!("Unwritable. Error: {e}");
        return Err(HyprvisorError::StreamError);
    }

    match stream.try_write(content.as_bytes()) {
        Ok(len) if len == content.len() => {
            log::debug!("Message: {content}");
            log::debug!("{len} bytes written");
            Ok(())
        }
        Ok(len) => {
            log::warn!("Can't write all message. {len} bytes written");
            Err(HyprvisorError::StreamError)
        }
        Err(e) => {
            log::error!("Can't write to stream. Error: {e}");
            Err(HyprvisorError::StreamError)
        }
    }
}

#[allow(unused)]
pub async fn try_write_multiple(
    stream: &UnixStream,
    content: &str,
    max_try: usize,
) -> HyprvisorResult<()> {
    for attempt in 0..max_try {
        match try_write(stream, content).await {
            Ok(_) => {
                return Ok(());
            }
            Err(_) => {
                log::warn!("Retry {}/{}", attempt + 1, max_try);
                continue;
            }
        }
    }

    log::error!("Out of attempt");
    Err(HyprvisorError::StreamError)
}

pub async fn try_read(stream: &UnixStream) -> HyprvisorResult<String> {
    if let Err(e) = stream.readable().await {
        log::error!("Unreadable. Error: {e}");
        return Err(HyprvisorError::StreamError);
    }

    let mut buffer = vec![0; 8192];
    match stream.try_read(&mut buffer) {
        Ok(len) if len > 2 => {
            let response = String::from_utf8_lossy(&buffer[0..len]).to_string();
            log::debug!("Success: {response}");
            Ok(response)
        }
        Ok(_) => {
            log::error!("Invalid message");
            Err(HyprvisorError::StreamError)
        }
        Err(e) => {
            log::error!("Can't read from stream. Error: {e}");
            Err(HyprvisorError::StreamError)
        }
    }
}

pub async fn try_read_multiple(stream: &UnixStream, max_try: usize) -> HyprvisorResult<String> {
    for attempt in 0..max_try {
        match try_read(stream).await {
            Ok(response) => {
                return Ok(response);
            }
            Err(_) => {
                log::warn!("Retry {}/{}", attempt + 1, max_try);
                continue;
            }
        }
    }

    log::error!("Out of attempt");
    Err(HyprvisorError::StreamError)
}

pub async fn try_connect_and_wait(
    socket_path: &str,
    content: &str,
    max_try: usize,
) -> HyprvisorResult<String> {
    let stream = try_connect(socket_path, 3, 100).await?;

    for attempt in 0..max_try {
        try_write(&stream, content).await?;
        match try_read(&stream).await {
            Ok(response) => {
                return Ok(response);
            }
            Err(_) => {
                log::warn!("Retry attempt: {}/{}", attempt + 1, max_try);
            }
        }
    }

    log::error!("Maximum retry attempts reached");
    Err(HyprvisorError::StreamError)
}

pub async fn try_write_and_wait(
    stream: &UnixStream,
    content: &str,
    max_try: usize,
) -> HyprvisorResult<String> {
    for attempt in 0..max_try {
        try_write(stream, content).await?;
        match try_read(stream).await {
            Ok(response) => {
                return Ok(response);
            }
            Err(_) => {
                log::warn!("Retry attempt: {}/{}", attempt + 1, max_try);
            }
        }
    }

    log::error!("Maximum retry attempts reached");
    Err(HyprvisorError::StreamError)
}
