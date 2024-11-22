use super::{
    message::HyprvisorMessage, HyprvisorReadSock, HyprvisorRequestResponse, HyprvisorWriteSock,
};
use crate::{
    error::{HyprvisorError, HyprvisorResult},
    global::BUFFER_SIZE,
};

use tokio::net::{unix::OwnedWriteHalf, UnixStream};

impl HyprvisorReadSock for UnixStream {
    async fn read_bytes(&self, buffer: &mut [u8]) -> HyprvisorResult<usize> {
        if let Err(e) = self.readable().await {
            log::error!("Unreadable. Error: {e}");
            return Err(HyprvisorError::IpcError);
        }

        match self.try_read(buffer) {
            Ok(len) if len > 0 => Ok(len),
            Ok(_) => {
                log::error!("Failed to read.");
                Err(HyprvisorError::IpcError)
            }
            Err(e) => {
                log::info!("Can't read from stream. Error: {e}");
                Err(HyprvisorError::IpcError)
            }
        }
    }

    async fn try_read_bytes(&self, buffer: &mut [u8], max_attempt: u8) -> HyprvisorResult<usize> {
        for attempt in 0..max_attempt {
            match self.read_bytes(buffer).await {
                Ok(len) => return Ok(len),
                Err(_) => log::warn!("Retry {}/{}", attempt + 1, max_attempt),
            }
        }
        log::error!("Out of attempt");
        Err(HyprvisorError::IpcError)
    }

    async fn read_message(&self) -> HyprvisorResult<HyprvisorMessage> {
        let mut buffer = vec![0; *BUFFER_SIZE];
        match self.read_bytes(&mut buffer).await {
            Ok(len) if len > 0 => buffer[..len].try_into(),
            Ok(_) => {
                log::error!("Invalid message");
                Err(HyprvisorError::IpcError)
            }
            Err(e) => {
                log::info!("Can't read from stream. Error: {e}");
                Err(HyprvisorError::IpcError)
            }
        }
    }

    async fn try_read_message(&self, max_attempt: u8) -> HyprvisorResult<HyprvisorMessage> {
        for attempt in 0..max_attempt {
            match self.read_message().await {
                Ok(message) => return Ok(message),
                Err(_) => log::warn!("Retry {}/{}", attempt + 1, max_attempt),
            }
        }
        log::error!("Out of attempt");
        Err(HyprvisorError::IpcError)
    }
}

impl HyprvisorWriteSock for UnixStream {
    async fn write_bytes(&self, buffer: &[u8]) -> HyprvisorResult<usize> {
        if let Err(e) = self.writable().await {
            log::error!("Unwritable. Error: {e}");
            return Err(HyprvisorError::IpcError);
        }

        match self.try_write(buffer) {
            Ok(len) if len == buffer.len() => {
                log::debug!("{len} bytes were written.");
                Ok(len)
            }
            Ok(len) => {
                log::warn!("Can't write all message. {len} bytes were written.");
                Err(HyprvisorError::IpcError)
            }
            Err(e) => {
                log::info!("Can't write to stream. Error: {e}");
                Err(HyprvisorError::IpcError)
            }
        }
    }

    async fn try_write_bytes(&self, buffer: &[u8], max_attempt: u8) -> HyprvisorResult<usize> {
        for attempt in 0..max_attempt {
            match self.write_bytes(buffer).await {
                Ok(len) => return Ok(len),
                Err(_) => log::warn!("Retry {}/{}", attempt + 1, max_attempt),
            }
        }
        log::error!("Out of attempt.");
        Err(HyprvisorError::IpcError)
    }

    async fn write_message(&self, message: HyprvisorMessage) -> HyprvisorResult<usize> {
        let buffer: Vec<u8> = message.into();
        match self.write_bytes(&buffer).await {
            Ok(len) => Ok(len),
            Err(_) => Err(HyprvisorError::IpcError),
        }
    }

    async fn try_write_message(
        &self,
        message: &HyprvisorMessage,
        max_attempt: u8,
    ) -> HyprvisorResult<usize> {
        for attempt in 0..max_attempt {
            match self.write_message(message.clone()).await {
                Ok(len) => return Ok(len),
                Err(_) => log::warn!("Retry {}/{}", attempt + 1, max_attempt),
            }
        }
        log::error!("Out of attempt.");
        Err(HyprvisorError::IpcError)
    }
}

impl HyprvisorRequestResponse for UnixStream {
    async fn send_and_receive_bytes(
        &self,
        data: &[u8],
        buffer: &mut [u8],
    ) -> HyprvisorResult<usize> {
        self.write_bytes(data).await?;
        self.read_bytes(buffer).await
    }

    async fn try_send_and_receive_bytes(
        &self,
        data: &[u8],
        buffer: &mut [u8],
        max_attempt: u8,
    ) -> HyprvisorResult<usize> {
        self.try_write_bytes(data, max_attempt).await?;
        self.read_bytes(buffer).await
    }

    async fn send_and_receive_message(
        &self,
        message: HyprvisorMessage,
    ) -> HyprvisorResult<HyprvisorMessage> {
        self.write_message(message).await?;
        self.read_message().await
    }

    async fn try_send_and_receive_message(
        &self,
        message: &HyprvisorMessage,
        max_attempt: u8,
    ) -> HyprvisorResult<HyprvisorMessage> {
        self.try_write_message(message, max_attempt).await?;
        self.read_message().await
    }
}

impl HyprvisorWriteSock for OwnedWriteHalf {
    async fn write_bytes(&self, buffer: &[u8]) -> HyprvisorResult<usize> {
        if let Err(e) = self.writable().await {
            log::error!("Unwritable. Error: {e}");
            return Err(HyprvisorError::IpcError);
        }

        match self.try_write(buffer) {
            Ok(len) if len == buffer.len() => {
                log::debug!("{len} bytes were written.");
                Ok(len)
            }
            Ok(len) => {
                log::warn!("Can't write all message. {len} bytes were written.");
                Err(HyprvisorError::IpcError)
            }
            Err(e) => {
                log::info!("Can't write to stream. Error: {e}");
                Err(HyprvisorError::IpcError)
            }
        }
    }

    async fn try_write_bytes(&self, buffer: &[u8], max_attempt: u8) -> HyprvisorResult<usize> {
        for attempt in 0..max_attempt {
            match self.write_bytes(buffer).await {
                Ok(len) => return Ok(len),
                Err(_) => log::warn!("Retry {}/{}", attempt + 1, max_attempt),
            }
        }
        log::error!("Out of attempt.");
        Err(HyprvisorError::IpcError)
    }

    async fn write_message(&self, message: HyprvisorMessage) -> HyprvisorResult<usize> {
        let buffer: Vec<u8> = message.into();
        match self.write_bytes(&buffer).await {
            Ok(len) => Ok(len),
            Err(_) => Err(HyprvisorError::IpcError),
        }
    }

    async fn try_write_message(
        &self,
        message: &HyprvisorMessage,
        max_attempt: u8,
    ) -> HyprvisorResult<usize> {
        for attempt in 0..max_attempt {
            match self.write_message(message.clone()).await {
                Ok(len) => return Ok(len),
                Err(_) => log::warn!("Retry {}/{}", attempt + 1, max_attempt),
            }
        }
        log::error!("Out of attempt.");
        Err(HyprvisorError::IpcError)
    }
}
