use amane::{Center, Color, Monitor, Parent, Rectangle, Row, Service, Text, Widget, Workspace, Workspaces};

pub fn view(monitor: &Monitor) -> Row {
    let workspaces = Workspaces::read();

    let mut buttons: Vec<Box<dyn Widget>> = Vec::new();

    for workspace in workspaces.list() {
        // only this monitor's workspaces
        if workspace.output() != Some(monitor.name.as_str()) {
            continue;
        }

        buttons.push(Box::new(button(workspace)));
    }

    Row::new(buttons).gap(4.0)
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
        .align_child(Center, Center)
        .on_click(move |_| Workspaces::focus(id))
        .child(Text::new(workspace.index().to_string()).color(Color::WHITE))
}
