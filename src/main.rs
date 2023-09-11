use hyprland::data::{Client, Clients, Monitors, Workspace, Workspaces};
use hyprland::dispatch::*;
use hyprland::event_listener::EventListenerMutable as EventListener;
use hyprland::keyword::*;
use hyprland::prelude::*;
use hyprland::shared::WorkspaceType;

fn main() -> hyprland::Result<()> {
    // let monitors = Monitors::get()?;
    // println!("monitors: {monitors:#?}");
    //
    // let workspaces = Workspaces::get()?;
    // println!("workspaces: {workspaces:#?}");

    // Print workspace change
    let mut event_listener = EventListener::new();

    event_listener.add_workspace_change_handler(|id, _| match id {
        WorkspaceType::Regular(workspace) => {
            println!("Active workspace changed: {}", workspace);
        }
        _ => {
            println!("Invalid")
        }
    });

    event_listener.add_active_window_change_handler(|data, _| match data {
        Some(window_data) => {
            println!("Window focus changed: {}", window_data.window_title)
        }
        None => println!("..."),
    });

    event_listener.start_listener()
}
