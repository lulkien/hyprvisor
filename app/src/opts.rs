use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Opts {
    action: Action,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Parser)]
pub enum Action {
    #[command(name = "daemon", alias = "d")]
    Daemon,

    #[command(flatten)]
    Command(ServerCommand),

    #[command(flatten)]
    Listen(Subscription),
}

#[derive(Debug, Deserialize, PartialEq, Serialize, Subcommand)]
pub enum ServerCommand {
    #[command(name = "kill", alias = "k")]
    Kill,

    #[command(name = "restart", alias = "r")]
    Restart,
}

#[derive(Debug, Deserialize, PartialEq, Serialize, Subcommand)]
pub enum Subscription {
    #[command(name = "workspaces")]
    Workspaces,

    #[command(name = "window")]
    Window,
}

impl Opts {
    pub fn from_env() -> Self {
        let action = Action::parse();
        action.into()
    }
}

impl From<Action> for Opts {
    fn from(action: Action) -> Self {
        Opts { action }
    }
}
