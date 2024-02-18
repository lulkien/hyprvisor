use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Opts {
    pub debug: bool,
    pub action: Action,
}

#[derive(Debug, Parser, Serialize, Deserialize, PartialEq)]
#[clap(author = "LulKien")]
#[clap(version, about)]
struct RawOpts {
    /// Write out debug logs. (To read the logs, run `hyprvisor debug`).
    #[arg(long = "debug", global = true)]
    debug: bool,

    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand, Debug, Serialize, Deserialize, PartialEq)]
pub enum Action {
    #[command(name = "daemon", alias = "d")]
    Daemon,

    #[command(flatten)]
    Command(ServerCommand),

    #[command(flatten)]
    Listen(Subscription),
}

#[derive(Debug, Deserialize, Serialize, Subcommand, PartialEq)]
pub enum ServerCommand {
    #[command(name = "ping", alias = "p")]
    Ping,

    #[command(name = "kill", alias = "k")]
    Kill,
}

#[derive(Debug, Deserialize, Serialize, Subcommand, PartialEq)]
pub enum Subscription {
    #[command(name = "workspaces", alias = "ws")]
    Workspaces { fix_workspace: Option<u32> },

    #[command(name = "window", alias = "w")]
    Window { title_length: Option<u32> },
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
            debug: raw_opts.debug,
            action: raw_opts.action,
        }
    }
}
