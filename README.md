 # Hyprvisor

[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-cyan.svg)](http://unlicense.org/)
[![Hyprland](https://img.shields.io/badge/Made%20for-Hyprland-blue.svg)](https://github.com/hyprwm/Hyprland)
![Build](https://github.com/lulkien/hyprvisor/actions/workflows/stable_build.yml/badge.svg)

## Overview

A Rust-based server and client designed for monitoring [Hyprland](https://github.com/hyprwm/Hyprland)'s workspace and active window. The server emits real-time JSON-formatted information, facilitating efficient communication with multiple clients.

## How to install?

1. **Install Rustup:**

Follow the official instructions to install [Rustup](https://rustup.rs/), the Rust toolchain manager.

Or, if you are using Arch Linux, then here you go:

```bash
sudo pacman -S rustup
```

2. **Install Rust toolchain:**

```bash
rustup install nightly
```

or

```bash
rustup install stable
```

3. **Clone the Repository:**

```bash
git clone https://github.com/lulkien/hyprvisor.git
```

4. **Build and Install**

Change directory into the repo and run:

```bash
cd app
cargo install --path .
```

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

- [$HOME/.config/hypr/scripts/start-hyprvisor.sh](https://github.com/lulkien/dotfiles/blob/master/config/hypr/scripts/start-hyprvisor.sh)

  ```bash
  #!/usr/bin/env bash
  
  ~/.cargo/bin/hyprvisor kill
  sleep 0.2
  ~/.cargo/bin/hyprvisor daemon
  ```

- [$HOME/.config/hypr/hyprland.conf](https://github.com/lulkien/dotfiles/blob/master/config/hypr/hyprland.conf)
  ```bash
  # Your config
  ...
  # Start hyprvisor server, this is a backend for some EWW widgets
  exec-once = ~/.config/hypr/scripts/start-hyprvisor.sh

  # Start EWW
  exec-once = eww daemon
  exec-open = <start other eww daemon>
  exec-once = <open your eww widgets>
  ...
  # Your config
  ```
- [$HOME/.config/eww/modules/workspaces.yuck](https://github.com/lulkien/dotfiles/blob/master/config/eww/modules/workspaces.yuck)
  ```yuck
  ;; Listen to Hyprland's workspace
  (deflisten workspaces :initial "[]"
    "sh -c '~/.cargo/bin/hyprvisor workspaces 10'")

  (defwidget workspaces-widget []
    (box :class "workspaces-widget"
      (for ws in workspaces
        (eventbox
          :onclick "hyprctl dispatch workspace ${ws.id}"
          (box :class "workspace-button-${ws.active ? "active" : "deactive"}"
            :tooltip "Workspace ${ws.id}"
            (label
              :text "${ws.occupied ? ws.active ? "" : "󰻃"
                                   : ws.active ? "" : "" }"
            )
          )
        )
      )
    )
  )
  ```
Comprehensive configurations and further details can be found within the provided dotfiles repository.

## Disclaimer:

This tool is currently in active development and should be considered a work in progress. The current version provides support for monitoring workspace and window information. Please note that additional features may be implemented in future releases, subject to available development time. Users are advised that there may be existing issues, and we appreciate your understanding as we work to enhance and refine the tool further. Feedback and contributions are welcome as we strive to improve its functionality and reliability.

## Legacy Stable Version:

For users seeking a stable and straightforward experience, a legacy version is available on the `legacy-stable` branch. This version, while simpler in design, provides reliable functionality. Feel free to check out this branch to explore the stable release and experience a more straightforward usage. Your feedback on this legacy version is valuable as we continue to evolve and enhance the tool. To try the legacy stable version, use the following command:

```bash
git checkout legacy-stable
```

Explore the simplicity and stability of this legacy release, and don't hesitate to provide feedback or report any issues you may encounter.

## License

This project is released into the public domain under the [Unlicense](https://unlicense.org). See the [UNLICENSE](https://github.com/lulkien/dotfiles/blob/master/UNLICENSE) file for details.

Happy ricing! ✨
