pub(super) mod event;
pub(super) mod socket_type;
pub(super) mod window;
pub(super) mod workspace;

use crate::error::HyprvisorResult;

pub use event::{HyprEvent, HyprEventList};
pub use socket_type::HyprSocketType;
pub use window::HyprWindowInfo;
pub use workspace::HyprWorkspaceInfo;

pub trait FormattedInfo {
    fn to_formatted_json(self, extra_data: &u32) -> HyprvisorResult<String>;
}
