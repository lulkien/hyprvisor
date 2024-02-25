use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Opts {
    pub verbose: bool,
    pub action: Action,
}

#[derive(Debug, Parser, Serialize, Deserialize, PartialEq)]
#[clap(author = "LulKien")]
#[clap(version, about)]
struct RawOpts {
    /// Run hyprvisor with log level DEBUG.
    #[arg(long = "verbose", short = 'v')]
    verbose: bool,

    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand, Debug, Serialize, Deserialize, PartialEq)]
pub enum Action {
    #[command(name = "daemon", alias = "d")]
    Daemon,

    #[command(flatten)]
    Command(CommandOpts),

    #[command(flatten)]
    Listen(SubscribeOpts),
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Subcommand)]
pub enum CommandOpts {
    #[command(name = "ping", alias = "p")]
    Ping,

    #[command(name = "kill", alias = "k")]
    Kill,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Subcommand)]
pub enum SubscribeOpts {
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
            verbose: raw_opts.verbose,
            action: raw_opts.action,
        }
    }
}
