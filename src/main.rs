// std of course
use std::env;

// some hyprland stuffs
use hyprland::data::{Workspace, Workspaces};
use hyprland::event_listener::AsyncEventListener;
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

#[tokio::main]
async fn main() -> hyprland::Result<()> {
    let args: Vec<String> = env::args().collect();

    // Verify arguments
    if args.len() != 2 {
        let error_msg: String = match args.len() {
            1 => "Need an argument: workspaces.".to_string(),
            _ => "Too many arguments.".to_string(),
        };
        eprintln!("Error: {}", error_msg);
        return Ok(());
    }

    // Print data
    // let mut event_listener = EventListener::new();
    // match args[1].as_str() {
    //     "workspace" => {
    //         // Just init
    //         let active_workspace: String = match Workspace::get_active() {
    //             Ok(value) => value.id.to_string(),
    //             Err(_) => "[]".to_string(),
    //         };
    //         let workspace_json_init = get_workspace_json(&active_workspace);
    //         println!("{workspace_json_init}");
    //
    //         // Subcribe event
    //         event_listener.add_workspace_change_handler(|data, _| match data {
    //             WorkspaceType::Regular(active_workspace) => {
    //                 let workspace_json = get_workspace_json(&active_workspace);
    //                 println!("{}", workspace_json);
    //             }
    //             _ => {
    //                 println!("[]");
    //             }
    //         });
    //     }
    //     "window" => {
    //         // Just init
    //         let active_window: String = match Client::get_active() {
    //             Ok(client_opt) => match client_opt {
    //                 Some(client) => client.title,
    //                 None => "...".to_string(),
    //             },
    //             Err(_) => "...".to_string(),
    //         };
    //         println!("{active_window}");
    //
    //         // Subcribe event
    //         event_listener.add_active_window_change_handler(|event, _| match event {
    //             Some(event_data) => {
    //                 println!("{}", event_data.window_title);
    //             }
    //             None => println!("..."),
    //         });
    //     }
    //     _ => {
    //         eprintln!("Error: Invalid argument '{}'", args[1]);
    //         return Ok(());
    //     }
    // }
    // event_listener.start_listener()

    let mut hyprvisor = AsyncEventListener::new();

    match args[1].as_str() {
        "workspaces" => {
            // Just init
            let active_workspace = get_active_workspace_id().await;
            let workspaces_data_init = get_workspace_json_async(&active_workspace).await;
            println!("{}", workspaces_data_init);

            // Subcribe event
            hyprvisor.add_workspace_change_handler(async_closure!(move |wst| match wst {
                WorkspaceType::Regular(active_workspace) => {
                    let active_id = match active_workspace.parse::<i32>() {
                        Ok(id) => id,
                        Err(_) => 0,
                    };
                    let current_workspaces_data = get_workspace_json_async(&active_id).await;
                    println!("{current_workspaces_data}");
                }
                _ => println!("[]"),
            }));

            // update workspaces when open/close a new window
            hyprvisor.add_window_open_handler(async_closure!(|_| {
                let active_workspace = get_active_workspace_id().await;
                let workspaces_data_update = get_workspace_json_async(&active_workspace).await;
                println!("{}", workspaces_data_update);
            }));

            hyprvisor.add_window_close_handler(async_closure!(|_| {
                let active_workspace = get_active_workspace_id().await;
                let workspaces_data_update = get_workspace_json_async(&active_workspace).await;
                println!("{}", workspaces_data_update);
            }));
        }
        _ => {
            eprintln!("Error: Invalid argument '{}'", args[1]);
            return Ok(());
        }
    }

    hyprvisor.start_listener_async().await
}

#[allow(dead_code)]
fn shorten_string(input: &str) -> String {
    if input.len() <= 40 {
        input.to_string()
    } else {
        let mut result = input[..37].to_string();
        result.push_str("...");
        result
    }
}

#[allow(dead_code)]
fn get_workspace_json(active_workspace: &WorkspaceId) -> String {
    let workspaces = match Workspaces::get() {
        Ok(data) => data,
        Err(_) => return "[]".to_string(),
    };
    let occupied_workspaces: Vec<WorkspaceId> = workspaces
        .iter()
        .filter(|ws| ws.windows > 0)
        .map(|ws| ws.id)
        .collect();

    let workspaces_info: Vec<WorkspaceInfo> = (1..=10)
        .map(|index| WorkspaceInfo {
            id: index,
            active: index == *active_workspace,
            occupied: occupied_workspaces.contains(&index),
        })
        .collect();
    match serde_json::to_string(&workspaces_info) {
        Ok(data) => data,
        Err(_) => "[]".to_string(),
    }
}

async fn get_workspace_json_async(active_workspace: &WorkspaceId) -> String {
    let workspaces = match Workspaces::get_async().await {
        Ok(data) => data,
        Err(_) => return "[]".to_string(),
    };
    let occupied_workspaces: Vec<WorkspaceId> = workspaces
        .iter()
        .filter(|ws| ws.windows > 0)
        .map(|ws| ws.id)
        .collect();

    let workspace_info: Vec<WorkspaceInfo> = (1..=10)
        .map(|idx| WorkspaceInfo {
            id: idx,
            active: idx == *active_workspace,
            occupied: occupied_workspaces.contains(&idx),
        })
        .collect();
    match serde_json::to_string(&workspace_info) {
        Ok(data) => data,
        Err(_) => "[]".to_string(),
    }
}

async fn get_active_workspace_id() -> WorkspaceId {
    match Workspace::get_active_async().await {
        Ok(workspace) => workspace.id,
        Err(_) => 0,
    }
}
