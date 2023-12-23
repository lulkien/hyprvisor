# Hyprvisor

[![Hyprland](https://img.shields.io/badge/Made%20for-Hyprland-blue)](https://github.com/hyprwm/Hyprland)

A command-line interface (CLI) tool in Rust, powered by the [hyprland-rs](https://github.com/hyprland-community/hyprland-rs), dedicated to monitoring [Hyprland](https://github.com/hyprwm/Hyprland)'s workspace and emitting JSON-formatted information.

## Installation

### Compiling from Source

Install dependencies:

* cargo \*

Run these commands:

```bash
  git clone https://github.com/lulkien/hyprvisor
  cd hyprvisor
  cargo install --path .
```

## Output

```console
  [{"id":1,"active":true,"occupied":true},{"id":2,"active":false,"occupied":false},{"id":3,"active":false,"occupied":false},{"id":4,"active":false,"occupied":false},{"id":5,"active":false,"occupied":false},{"id":6,"active":false,"occupied":false},{"id":7,"active":false,"occupied":false},{"id":8,"active":false,"occupied":false},{"id":9,"active":false,"occupied":false},{"id":10,"active":false,"occupied":false}]
```

## Usage

This tool can be used in [eww](https://github.com/elkowar/eww)'s widget.

### Example

[workspace.yuck](https://github.com/QuantaRicer/eww/blob/master/modules/workspaces.yuck)

```yuck
  ;; Listener
  (deflisten workspaces :initial "[]"
      "bash -c '~/.cargo/bin/hyprvisor workspaces'")
  
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

## Disclaimer

Note: This tool is a work in progress. As of the current version, it supports only one feature: listening to changes in the workspace. Future updates may include additional features, but at the moment, the functionality is limited to this singular aspect. Use this tool with the understanding that it is not a fully completed product, and some features may be added or modified in subsequent releases. Your feedback and contributions are welcome as the tool evolves.

Thank you for your understanding.
