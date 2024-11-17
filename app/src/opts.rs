use std::fmt::Display;

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

use crate::error::HyprvisorError;

#[derive(Deserialize, Serialize, PartialEq)]
pub struct Opts {
    pub verbose: bool,
    pub action: Action,
}

#[derive(Parser, Serialize, Deserialize, PartialEq)]
#[clap(author = "LulKien")]
#[clap(version, about)]
struct RawOpts {
    /// Run hyprvisor with log level DEBUG.
    #[arg(long = "verbose", short = 'v')]
    verbose: bool,

    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand, Serialize, Deserialize, PartialEq)]
pub enum Action {
    #[command(name = "daemon", alias = "d")]
    Daemon,

    #[command(flatten)]
    Command(CommandOpts),

    #[command(flatten)]
    Listen(SubscribeOpts),
}

#[derive(Clone, Copy, PartialEq, Deserialize, Serialize, Subcommand)]
pub enum CommandOpts {
    #[command(name = "ping", alias = "p")]
    Ping,

    #[command(name = "kill", alias = "k")]
    Kill,
}

#[derive(Clone, Copy, PartialEq, Deserialize, Serialize, Subcommand)]
pub enum SubscribeOpts {
    #[command(name = "workspaces", alias = "ws")]
    Workspaces { fix_workspace: Option<u32> },

    #[command(name = "window", alias = "w")]
    Window { title_length: Option<u32> },

    #[command(name = "wireless", alias = "wl")]
    Wireless { ssid_length: Option<u32> },
}

impl Opts {
    pub fn from_env() -> Self {
        let raw_opts = RawOpts::parse();
        raw_opts.into()
    }
}

impl From<RawOpts> for Opts {
    fn from(raw_opts: RawOpts) -> Self {
        Opts {
            verbose: raw_opts.verbose,
            action: raw_opts.action,
        }
    }
}

impl From<CommandOpts> for u8 {
    fn from(opts: CommandOpts) -> Self {
        opts as u8
    }
}

impl TryFrom<u8> for CommandOpts {
    type Error = HyprvisorError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CommandOpts::Ping),
            1 => Ok(CommandOpts::Kill),
            _ => Err(HyprvisorError::ParseError),
        }
    }
}

impl Display for CommandOpts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandOpts::Ping => write!(f, "Ping"),
            CommandOpts::Kill => write!(f, "Kill"),
        }
    }
}
