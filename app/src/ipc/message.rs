use crate::{
    application::types::ClientInfo,
    error::{HyprvisorError, HyprvisorResult},
    hyprland::types::{HyprWindowInfo, HyprWorkspaceInfo},
    opts::CommandOpts,
};

const MESSAGE_TYPE_LEN: usize = size_of::<MessageType>();
const MESSAGE_HEADER_LEN: usize = size_of::<usize>();

#[derive(Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageType {
    Command = 0,
    Subscription = 1,
    Response = 2,
}

impl From<MessageType> for u8 {
    fn from(value: MessageType) -> Self {
        value as u8
    }
}

impl TryFrom<u8> for MessageType {
    type Error = HyprvisorError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MessageType::Command),
            1 => Ok(MessageType::Subscription),
            2 => Ok(MessageType::Response),
            _ => Err(HyprvisorError::ParseError),
        }
    }
}

#[derive(Clone)]
pub struct HyprvisorMessage {
    pub message_type: MessageType,
    pub header: usize, // size of payload
    pub payload: Vec<u8>,
}

impl From<CommandOpts> for HyprvisorMessage {
    fn from(opts: CommandOpts) -> HyprvisorMessage {
        HyprvisorMessage {
            message_type: MessageType::Command,
            header: size_of::<u8>(),
            payload: vec![opts.into()],
        }
    }
}

impl From<ClientInfo> for HyprvisorMessage {
    fn from(info: ClientInfo) -> HyprvisorMessage {
        HyprvisorMessage {
            message_type: MessageType::Subscription,
            header: ClientInfo::byte_size(),
            payload: info.into(),
        }
    }
}

impl From<&str> for HyprvisorMessage {
    fn from(data: &str) -> Self {
        HyprvisorMessage {
            message_type: MessageType::Response,
            header: data.len(),
            payload: data.as_bytes().to_vec(),
        }
    }
}

impl From<HyprvisorMessage> for Vec<u8> {
    fn from(message: HyprvisorMessage) -> Self {
        let mut buffer = Vec::new();

        buffer.push(u8::from(message.message_type));
        buffer.extend_from_slice(&message.header.to_le_bytes());
        buffer.extend_from_slice(&message.payload);

        buffer
    }
}

impl TryFrom<&[u8]> for HyprvisorMessage {
    type Error = HyprvisorError;
    fn try_from(buffer: &[u8]) -> Result<Self, Self::Error> {
        let metadata_len: usize = MESSAGE_TYPE_LEN + MESSAGE_HEADER_LEN;

        if buffer.len() < metadata_len {
            return Err(HyprvisorError::ParseError);
        }

        let message_type: MessageType = MessageType::try_from(buffer[0])?;
        let header: usize = usize::from_le_bytes(
            buffer[1..metadata_len]
                .try_into()
                .map_err(|_| HyprvisorError::ParseError)?,
        );

        if buffer.len() < (metadata_len + header) {
            return Err(HyprvisorError::ParseError);
        }

        let payload = buffer[metadata_len..(metadata_len + header)].to_vec();
        Ok(HyprvisorMessage {
            message_type,
            header,
            payload,
        })
    }
}

impl TryFrom<HyprWindowInfo> for HyprvisorMessage {
    type Error = HyprvisorError;
    fn try_from(window_info: HyprWindowInfo) -> HyprvisorResult<HyprvisorMessage> {
        let payload: Vec<u8> = bincode::serialize(&window_info)?;
        Ok(HyprvisorMessage {
            message_type: MessageType::Response,
            header: payload.len(),
            payload,
        })
    }
}

impl TryFrom<Vec<HyprWorkspaceInfo>> for HyprvisorMessage {
    type Error = HyprvisorError;
    fn try_from(workspaces: Vec<HyprWorkspaceInfo>) -> HyprvisorResult<HyprvisorMessage> {
        let payload: Vec<u8> = bincode::serialize(&workspaces)?;
        Ok(HyprvisorMessage {
            message_type: MessageType::Response,
            header: payload.len(),
            payload,
        })
    }
}

impl HyprvisorMessage {
    pub fn is_valid(&self) -> bool {
        self.payload.len() == self.header
    }

    pub fn len(&self) -> usize {
        return MESSAGE_TYPE_LEN + MESSAGE_HEADER_LEN + self.header;
    }
}
