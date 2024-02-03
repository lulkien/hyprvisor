use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Listen to Hyprland's workspaces changed
    Workspace { fix: Option<u8> },
    /// Listen to focused Hyprland's window changed
    Window,
    /// Listen to sink volume changed
    SinkVolume,
    /// Listen to source volume changed
    SourceVolume,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy, Deserialize, Serialize)]
#[allow(unused)]
pub(crate) enum SubscriptionID {
    Workspace,
    Window,
    SinkVolume,
    SourceVolume,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct SubscriptionInfo {
    pub pid: u32,
    pub subscription_id: SubscriptionID,
}
