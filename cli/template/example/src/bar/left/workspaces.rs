use amane::{LayerWindow, Workspaces};

fn view() -> LayerWindow {
    let workspaces = Workspaces::read();

    let mut buttons: Vec<Box<dyn widget>> = Vec::new();
}
