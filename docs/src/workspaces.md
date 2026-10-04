# Workspaces

`Workspaces` lists your compositor's workspaces and can switch between them. It supports **niri**, **Hyprland** and **Sway**. On other compositors, the list stays empty.

It doesn't poll. It follows the compositor's event stream, so the bar updates the moment you switch.

## Workspace buttons

```rust
use amane::{App, Color, Full, LayerWindow, Parent, Rectangle, Row, Service, Text, Widget, Workspace, Workspaces};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let workspaces = Workspaces::read();

    let mut buttons: Vec<Box<dyn Widget>> = Vec::new();

    for workspace in workspaces.list() {
        buttons.push(Box::new(button(workspace)));
    }

    LayerWindow::new().width(Full).height(30.0).child(Row::new(buttons))
}

fn button(workspace: &Workspace) -> Rectangle {
    let fill = if workspace.focused() {
        "#89b4fa"
    } else if workspace.urgent() {
        "#f38ba8"
    } else {
        "#313244"
    };

    let id = workspace.id();

    Rectangle::new()
        .width(30.0)
        .height(Parent)
        .fill(fill)
        .on_click(move |_| Workspaces::focus(id))
        .child(Text::new(workspace.index().to_string()).color(Color::WHITE))
}
```

## Reference

`list()` gives every workspace, sorted by monitor, then by position on it. Each `Workspace` has:

| Function | Gives |
|---|---|
| `id()` | the compositor's id for it, which `Workspaces::focus` takes |
| `index()` | its position on its monitor, starting at 1; on Hyprland and Sway, the workspace's number (0 for a workspace with only a name) |
| `name()` | its name, or `None` for workspaces you never named |
| `output()` | the monitor it's on, like `"DP-1"`, or `None` |
| `active()` | `true` when it's the one shown on its monitor, even when another monitor has focus |
| `focused()` | `true` for the one workspace that has focus overall |
| `urgent()` | `true` when a window on it asks for attention; always `false` on Hyprland |
| `windows()` | how many windows are on it |

`Workspaces::focus(id)` switches to a workspace. It runs on a background thread and returns right away.

## Per-monitor bars

With `window_per_monitor`, show each bar only its own monitor's workspaces by comparing `output()` with the monitor's name ([Multiple Windows and Monitors](windows.md)):

```rust,ignore
fn bar(monitor: &Monitor) -> LayerWindow {
    let workspaces = Workspaces::read();

    for workspace in workspaces.list() {
        if workspace.output() != Some(monitor.name.as_str()) {
            continue;
        }

        // add a button
    }

    // ...
}
```
