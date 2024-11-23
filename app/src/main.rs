mod application;
mod bluetooth;
mod error;
mod global;
mod hyprland;
mod ipc;
mod opts;
mod wifi;

use crate::{
    error::HyprvisorResult,
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

    match opts.action {
        Action::Daemon => {
            application::server::start_server(level_filter).await?;
        }
        Action::Command(command) => {
            application::standalone::send_command(command, level_filter).await?;
        }
        Action::Listen(subscription) => {
            application::client::start_client(subscription, level_filter).await?;
        }
    };

    Ok(())
}
