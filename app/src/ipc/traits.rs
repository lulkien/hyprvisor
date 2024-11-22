use super::message::HyprvisorMessage;
use crate::error::{HyprvisorError, HyprvisorResult};

use std::time::Duration;
use tokio::{net::UnixStream, time::sleep};

#[allow(unused)]
pub trait HyprvisorReadSock {
    async fn read_bytes(&self, buffer: &mut [u8]) -> HyprvisorResult<usize>;
    async fn try_read_bytes(&self, buffer: &mut [u8], max_attempt: u8) -> HyprvisorResult<usize>;

    async fn read_message(&self) -> HyprvisorResult<HyprvisorMessage>;
    async fn try_read_message(&self, max_attempt: u8) -> HyprvisorResult<HyprvisorMessage>;
}

pub trait HyprvisorWriteSock {
    async fn write_bytes(&self, buffer: &[u8]) -> HyprvisorResult<usize>;
    async fn try_write_bytes(&self, buffer: &[u8], max_attempt: u8) -> HyprvisorResult<usize>;

    async fn write_message(&self, message: HyprvisorMessage) -> HyprvisorResult<usize>;
    async fn try_write_message(
        &self,
        message: &HyprvisorMessage,
        max_attempt: u8,
    ) -> HyprvisorResult<usize>;
}

#[allow(unused)]
pub trait HyprvisorRequestResponse {
    async fn send_and_receive_bytes(
        &self,
        data: &[u8],
        buffer: &mut [u8],
    ) -> HyprvisorResult<usize>;
    async fn try_send_and_receive_bytes(
        &self,
        data: &[u8],
        buffer: &mut [u8],
        max_attempt: u8,
    ) -> HyprvisorResult<usize>;

    async fn send_and_receive_message(
        &self,
        message: HyprvisorMessage,
    ) -> HyprvisorResult<HyprvisorMessage>;
    async fn try_send_and_receive_message(
        &self,
        message: &HyprvisorMessage,
        max_attempt: u8,
    ) -> HyprvisorResult<HyprvisorMessage>;
}

pub async fn connect_to_socket(
    socket_path: &str,
    max_attempt: u8,
    delay: u64,
) -> HyprvisorResult<UnixStream> {
    for attempt in 0..max_attempt {
        if let Ok(stream) = UnixStream::connect(socket_path).await {
            return Ok(stream);
        }
        log::debug!("Try connect: {} | Attempt: {}", socket_path, attempt + 1);
        sleep(Duration::from_millis(delay)).await;
    }

    log::warn!("Failed to connect to socket: {socket_path}");
    Err(HyprvisorError::IpcError)
}
