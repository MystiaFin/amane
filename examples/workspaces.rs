use amane::{
    App, Color, Full, Layer, LayerWindow, Parent, Rectangle, Row, Service, Text, Vertical, Widget,
    Workspace, Workspaces,
};

fn main() {
    App::new().window(view).run();
}

fn view() -> LayerWindow {
    let workspaces = Workspaces::read();

    let mut buttons: Vec<Box<dyn Widget>> = Vec::new();

    for workspace in workspaces.list() {
        buttons.push(Box::new(workspace_button(workspace)));
    }

    LayerWindow::new()
        .width(Full)
        .height(30.0)
        .anchor_vertical(Vertical::Top)
        .layer(Layer::Top)
        .child(Row::new(buttons))
}

fn workspace_button(workspace: &Workspace) -> Rectangle {
    let fill = if workspace.focused() {
        Color::BLUE
    } else if workspace.urgent() {
        Color::RED
    } else {
        Color::BLACK
    };

    let id = workspace.id();

    let label = workspace.index().to_string();

    Rectangle::new()
        .width(30.0)
        .height(Parent)
        .fill(fill)
        .on_click(move |_| Workspaces::focus(id))
        .child(Text::new(&label).size(20.0).color(Color::WHITE))
}
