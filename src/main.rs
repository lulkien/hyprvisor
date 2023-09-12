// std of course
use std::env;

// some hyprland stuffs
use hyprland::data::{Client, Workspace, Workspaces};
use hyprland::event_listener::EventListenerMutable as EventListener;
use hyprland::prelude::*;
use hyprland::shared::{WorkspaceId, WorkspaceType};

// json
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct WorkspaceInfo {
    id: WorkspaceId,
    active: bool,
    occupied: bool,
}

fn main() -> hyprland::Result<()> {
    let args: Vec<String> = env::args().collect();

    // Verify arguments
    if args.len() != 2 {
        let error_msg: String = match args.len() {
            1 => "Need an argument: --workspace, --workspace-init, --window, --window-init."
                .to_string(),
            _ => "Too many arguments.".to_string(),
        };
        eprintln!("Error: {}", error_msg);
        return Ok(());
    }

    // Print data
    let mut event_listener = EventListener::new();
    match args[1].as_str() {
        "--workspace-init" => {
            let active_workspace: String = match Workspace::get_active() {
                Ok(value) => value.id.to_string(),
                Err(_) => "[]".to_string(),
            };
            let init_string = get_workspace_json(&active_workspace);
            println!("{}", init_string);
            return Ok(());
        }
        "--workspace" => {
            event_listener.add_workspace_change_handler(|data, _| match data {
                WorkspaceType::Regular(active_workspace) => {
                    let workspace_json = get_workspace_json(&active_workspace);
                    println!("{}", workspace_json);
                }
                _ => {
                    println!("[]");
                }
            });
        }
        "--window-init" => {
            let active_window: String = match Client::get_active() {
                Ok(client_opt) => match client_opt {
                    Some(client) => client.title,
                    None => "...".to_string(),
                },
                Err(_) => "...".to_string(),
            };
            println!("{active_window}");
            return Ok(());
        }
        "--window" => {
            event_listener.add_active_window_change_handler(|event, _| match event {
                Some(event_data) => {
                    println!("{}", event_data.window_title);
                }
                None => println!("..."),
            });
        }
        _ => {
            eprintln!("Error: Invalid argument '{}'", args[1]);
            return Ok(());
        }
    }

    event_listener.start_listener()
}

fn get_workspace_json(active_workspace: &String) -> String {
    let workspaces = match Workspaces::get() {
        Ok(data) => data,
        Err(_) => return "[]".to_string(),
    };
    let occupied_workspaces: Vec<WorkspaceId> =
        workspaces.iter().map(|workspace| workspace.id).collect();

    let workspaces_info: Vec<WorkspaceInfo> = (1..=10)
        .map(|index| WorkspaceInfo {
            id: index,
            active: index.to_string() == *active_workspace,
            occupied: occupied_workspaces.contains(&index),
        })
        .collect();
    match serde_json::to_string(&workspaces_info) {
        Ok(data) => data,
        Err(_) => "[]".to_string(),
    }
}
