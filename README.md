 # Hyprvisor

[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-cyan.svg)](http://unlicense.org/)
[![Hyprland](https://img.shields.io/badge/Made%20for-Hyprland-blue.svg)](https://github.com/hyprwm/Hyprland)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/lulkien/hyprvisor/.github%2Fworkflows%2Fstable_build.yml)

## Overview

A Rust-based server and client designed for monitoring [Hyprland](https://github.com/hyprwm/Hyprland)'s workspace and active window. The server emits real-time JSON-formatted information, facilitating efficient communication with multiple clients.

## How to install?

### Build from source code

#### Install Rustup

Follow the official instructions to install [Rustup](https://rustup.rs/), the Rust toolchain manager.

Or, if you are using Arch Linux, then here you go:

```bash
sudo pacman -S rustup
```

#### Install Rust toolchain

```bash
rustup install stable
```

#### Clone the Repository

```bash
git clone https://github.com/lulkien/hyprvisor.git
```

#### Build and Install

Change directory into the repo and run:

```bash
cd hyprvisor
cargo install --path app/
```
### Use PKGBUILD

```bash
mkdir /tmp/hyprvisor
cd /tmp/hyprvisor
wget https://raw.githubusercontent.com/lulkien/hyprvisor/refs/heads/master/PKGBUILD
makepkg -si
```

### Use precompiled release

## How to use?

1. **Start the server after Hyprland:**
   
   Please run the server after Hyprland. Or else, it won't work.

2. **Integration the client with Elkowars Wacky Widgets:**
   
- The client can be used with [Elkowars Wacky Widgets](https://github.com/elkowar/eww).
- Usage:
  ```console
  Usage: hyprvisor [OPTIONS] <COMMAND>

  Commands:
    daemon
    ping
    kill
    workspaces
    window
    help        Print this message or the help of the given subcommand(s)

  Options:
    -v, --verbose  Run hyprvisor with log level DEBUG
    -h, --help     Print help
    -V, --version  Print version
  ```
- You can listen to a fixed number of workspaces with `hyprvisor workspaces <number>`
- You can also limit the length of the active window's title with `hyprvisor window <number>`

3. **Exploring Additional Uses:**
   
   You may discover other effective ways to use this tool. Experiment with its functionalities and explore how it can enhance your workflow.

### [My personal dotfiles](https://github.com/lulkien/dotfiles)

- [$HOME/.configs/hypr/subconfigs/hypr_startup.conf](https://github.com/lulkien/dotfiles/blob/master/configs/hypr/subconfigs/hypr_startup.conf)
  ```bash
  ...
  # Start hyprvisor and eww
  exec-once = hyprvisor daemon

  exec-once = eww daemon
  exec-once = eww open bar
  ...
  ```
- [$HOME/.configs/eww/widgets/bar/components/workspaces.yuck](https://github.com/lulkien/dotfiles/blob/master/configs/eww/widgets/bar/components/workspaces.yuck)
  ```yuck
  ;; Listener
  (deflisten WS_DATA :initial "[]"
    `hyprvisor ws 10`)

  (defwidget workspaces []
    (box :class "workspaces bar-container"
      (for ws in WS_DATA
        (button
          :onclick "hyprctl dispatch workspace ${ws.id}"
          :tooltip "Workspace ${ws.id}"
          (label
            :class "${ws.active ? "active" : "deactive"}"
            :text "${ws.occupied ? "" : ""}"
          )
        )
      )
    )
  )
  ```
- My EWW bar preview:
 ![Example widget](https://github.com/lulkien/hyprvisor/blob/48a6dcb0f1b6fe9927d9a2a2f4103c9b14af5eba/previews/example_eww_widget.png)
  
Comprehensive configurations and further details can be found within the provided dotfiles repository.

## Disclaimer:

This tool is currently in active development and should be considered a work in progress. The current version provides support for monitoring workspace and window information. Please note that additional features may be implemented in future releases, subject to available development time. Users are advised that there may be existing issues, and we appreciate your understanding as we work to enhance and refine the tool further. Feedback and contributions are welcome as we strive to improve its functionality and reliability.

## License

This project is released into the public domain under the [Unlicense](https://unlicense.org). See the [UNLICENSE](https://github.com/lulkien/dotfiles/blob/master/UNLICENSE) file for details.

Happy ricing! ✨
