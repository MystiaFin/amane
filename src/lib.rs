mod animation;
mod app;
mod files;
mod full;
mod graphics;
mod input;
mod ipc;
mod layer_window;
mod macros;
mod niri;
mod process;
mod services;
mod wayland;
mod widgets;

pub use animation::{Animation, Blend, Easing};
pub use app::App;
pub use files::watch_file;
pub use full::Full;
pub use graphics::{Cap, Color};
pub use input::{Button, Key, Scroll};
pub use ipc::{IpcCall, ipc_socket};
pub use layer_window::{
    Horizontal, Keyboard, Layer, LayerWindow, Margin, Vertical, WindowSize, Zone,
};
pub use process::{lines, output, spawn};
pub use services::{Service, Workspace, Workspaces};
pub use widgets::{
    Align, Arc, Canvas, Center, Circle, Column, End, Fill, Image, Justify, Line, Mask, Path,
    Radius, Rectangle, Row, Shadow, Shape, Size, Start, Text, Widget,
};

pub use widgets::Justify::{SpaceAround, SpaceBetween, SpaceEvenly};
pub use widgets::Size::Parent;
