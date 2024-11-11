mod application;
mod common_types;
mod error;
mod hyprland;
mod ipc;
mod iwd;
mod logger;
mod opts;
mod server;
mod utils;

use crate::{
    error::HyprvisorResult,
    logger::*,
    opts::{Action, Opts},
};

#[tokio::main]
async fn main() -> HyprvisorResult<()> {
    let opts = Opts::from_env();
    run(&opts).await?;

    Ok(())
}

async fn run(opts: &Opts) -> HyprvisorResult<()> {
    let level_filter = if opts.verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    match &opts.action {
        Action::Daemon => init_logger(LoggerType::Server, level_filter)?,
        Action::Command(_) => init_logger(LoggerType::Command, level_filter)?,
        _ => {}
    };

    match &opts.action {
        Action::Daemon => {
            todo!()
        }
        Action::Command(_command) => {
            todo!()
        }
        Action::Listen(subscription) => {
            application::client::start_client(subscription, level_filter).await?;
        }
    };

    Ok(())
}
