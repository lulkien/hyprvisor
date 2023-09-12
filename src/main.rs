use std::env;

use hyprland::data::Workspaces;
use hyprland::event_listener::EventListenerMutable as EventListener;
use hyprland::prelude::*;
use hyprland::shared::{HyprError, WorkspaceId, WorkspaceType};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct WorkspaceInfo {
    id: WorkspaceId,
    active: bool,
    occupied: bool,
}

fn main() -> hyprland::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        let error_message = match args.len() {
            0 => "Need an argument: workspace, window",
            1 => "Need an argument: workspace, window",
            _ => "Too many arguments",
        };
        return Err(HyprError::NotOkDispatch(error_message.to_string()));
    }

    // Print workspace change
    let mut event_listener = EventListener::new();
    match args[1].as_str() {
        "workspace" => {
            event_listener.add_workspace_change_handler(|data, _| match data {
                WorkspaceType::Regular(active_workspace) => {
                    let workspaces = Workspaces::get().unwrap();
                    let occupied_workspace: Vec<WorkspaceId> =
                        workspaces.iter().map(|workspace| workspace.id).collect();

                    let workspaces_info: Vec<WorkspaceInfo> = (1..=10)
                        .map(|index| WorkspaceInfo {
                            id: index,
                            active: index.to_string() == active_workspace,
                            occupied: occupied_workspace.contains(&index),
                        })
                        .collect();

                    let workspace_json = serde_json::to_string(&workspaces_info).unwrap();
                    println!("{}", workspace_json);
                }
                _ => {
                    println!("Invalid");
                }
            });
        }
        "window" => {
            event_listener.add_active_window_change_handler(|data, _| match data {
                Some(window_data) => {
                    println!("Window focus changed: {}", window_data.window_title);
                }
                None => println!("..."),
            });
        }
        _ => {
            eprintln!("Error: Invalid argument '{}'", args[1]);
            return Err(HyprError::NotOkDispatch("Invalid argument".to_string()));
        }
    }

    event_listener.start_listener()
}
