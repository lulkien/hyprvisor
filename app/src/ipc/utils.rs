use std::time::Duration;
use tokio::{net::UnixStream, time::sleep};

use crate::{
    error::{HyprvisorError, HyprvisorResult},
    global::BUFFER_SIZE,
};

pub trait HyprvisorSocket {
    async fn read_once(&self) -> HyprvisorResult<Vec<u8>>;
    async fn read_multiple(&self, max_attempt: u8) -> HyprvisorResult<Vec<u8>>;
    async fn read_once_buf(&self, buffer: &mut [u8]) -> HyprvisorResult<usize>;
    async fn read_multiple_buf(&self, buffer: &mut [u8], max_attempt: u8)
        -> HyprvisorResult<usize>;

    async fn write_once(&self, content: &[u8]) -> HyprvisorResult<()>;
    async fn write_multiple(&self, content: &[u8], max_attempt: u8) -> HyprvisorResult<()>;

    async fn write_and_read_once(&self, content: &[u8]) -> HyprvisorResult<Vec<u8>>;
    async fn write_and_read_multiple(
        &self,
        content: &[u8],
        max_attempt: u8,
    ) -> HyprvisorResult<Vec<u8>>;
}

impl HyprvisorSocket for UnixStream {
    async fn read_once(&self) -> HyprvisorResult<Vec<u8>> {
        if let Err(e) = self.readable().await {
            log::error!("Unreadable. Error: {e}");
            return Err(HyprvisorError::StreamError);
        }

        let mut buffer = vec![0; *BUFFER_SIZE];
        match self.try_read(&mut buffer) {
            Ok(len) if len > 0 => Ok(buffer[..len].to_vec()),
            Ok(_) => {
                log::error!("Invalid message");
                Err(HyprvisorError::StreamError)
            }
            Err(e) => {
                log::info!("Can't read from stream. Error: {e}");
                Err(HyprvisorError::StreamError)
            }
        }
    }

    async fn read_multiple(&self, max_attempt: u8) -> HyprvisorResult<Vec<u8>> {
        for attempt in 0..max_attempt {
            match self.read_once().await {
                Ok(response) => {
                    return Ok(response);
                }
                Err(_) => {
                    log::warn!("Retry {}/{}", attempt + 1, max_attempt);
                    continue;
                }
            }
        }

        log::error!("Out of attempt");
        Err(HyprvisorError::StreamError)
    }

    async fn read_once_buf(&self, buffer: &mut [u8]) -> HyprvisorResult<usize> {
        if let Err(e) = self.readable().await {
            log::error!("Unreadable. Error: {e}");
            return Err(HyprvisorError::StreamError);
        }

        match self.try_read(buffer) {
            Ok(len) if len > 0 => Ok(len),
            Ok(_) => {
                log::error!("Invalid message");
                Err(HyprvisorError::StreamError)
            }
            Err(e) => {
                log::info!("Can't read from stream. Error: {e}");
                Err(HyprvisorError::StreamError)
            }
        }
    }

    async fn read_multiple_buf(
        &self,
        buffer: &mut [u8],
        max_attempt: u8,
    ) -> HyprvisorResult<usize> {
        for attempt in 0..max_attempt {
            match self.read_once_buf(buffer).await {
                Ok(len) => {
                    return Ok(len);
                }
                Err(_) => {
                    log::warn!("Retry {}/{}", attempt + 1, max_attempt);
                    continue;
                }
            }
        }

        log::error!("Out of attempt");
        Err(HyprvisorError::StreamError)
    }

    async fn write_once(&self, content: &[u8]) -> HyprvisorResult<()> {
        if let Err(e) = self.writable().await {
            log::error!("Unwritable. Error: {e}");
            return Err(HyprvisorError::StreamError);
        }

        match self.try_write(content) {
            Ok(len) if len == content.len() => {
                log::debug!("Message: {}", String::from_utf8_lossy(content));
                log::debug!("{len} bytes written");
                Ok(())
            }
            Ok(len) => {
                log::warn!("Can't write all message. {len} bytes written");
                Err(HyprvisorError::StreamError)
            }
            Err(e) => {
                log::info!("Can't write to stream. Error: {e}");
                Err(HyprvisorError::StreamError)
            }
        }
    }

    async fn write_multiple(&self, content: &[u8], max_attempt: u8) -> HyprvisorResult<()> {
        for attempt in 0..max_attempt {
            match self.write_once(content).await {
                Ok(_) => {
                    return Ok(());
                }
                Err(_) => {
                    log::warn!("Retry {}/{}", attempt + 1, max_attempt);
                    continue;
                }
            }
        }

        log::error!("Out of attempt");
        Err(HyprvisorError::StreamError)
    }

    async fn write_and_read_once(&self, content: &[u8]) -> HyprvisorResult<Vec<u8>> {
        self.write_once(content).await?;
        self.read_once().await
    }

    async fn write_and_read_multiple(
        &self,
        content: &[u8],
        max_attempt: u8,
    ) -> HyprvisorResult<Vec<u8>> {
        for attempt in 0..max_attempt {
            match self.write_and_read_once(content).await {
                Ok(response) => return Ok(response),
                Err(_) => {
                    log::warn!("Retry {}/{}", attempt + 1, max_attempt);
                    continue;
                }
            }
        }

        log::error!("Maximum retry attempts reached");
        Err(HyprvisorError::StreamError)
    }
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
    Err(HyprvisorError::StreamError)
}
