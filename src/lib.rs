mod allocator;
mod animation;
mod app;
mod changes;
mod compositor;
mod dbus;
mod files;
mod frame;
mod full;
mod graphics;
mod input;
mod ipc;
mod layer_window;
mod macros;
mod monitor;
mod placement;
mod process;
mod services;
mod style;
mod timing;
mod wayland;
mod widgets;
mod window;

pub use animation::{Animation, Blend, Easing, request_frame};
pub use app::App;
pub use dbus::{Argument, Bus, Method, Signal, Value};
pub use files::watch_file;
pub use full::Full;
pub use graphics::{Cap, Color, Gradient, Weight};
pub use input::{Button, Cursor, Key, Point, Scroll};
pub use input::cursor_names::{
    Crosshair, Default, Grab, Grabbing, Move, NotAllowed, Pointer, ResizeBottom, ResizeBottomLeft,
    ResizeBottomRight, ResizeHorizontal, ResizeLeft, ResizeRight, ResizeTop, ResizeTopLeft,
    ResizeTopRight, ResizeVertical, Text, Wait,
};
pub use ipc::{IpcCall, ipc_socket};
pub use monitor::Monitor;
pub use layer_window::{
    Horizontal, InputArea, Keyboard, Layer, LayerWindow, Margin, Vertical, WindowSize, Zone,
};
pub use placement::{Align, Center, End, Justify, Padding, Size, Start};
pub use process::{lines, output, spawn};
pub use services::{
    AccessPoint, Action, Apps, Audio, Battery, Bluetooth, BluetoothDevice, Brightness, Cpu,
    DesktopApp, Link, Lock, Media, MediaPlayer, Memory, Network, Notification, Notifications,
    Palette, Service, Urgency, Workspace, Workspaces,
};
pub use style::{Fill, Image, Mask, Radius, Shadow};
pub use widgets::{
    Arc, Canvas, Circle, Column, Line, Path, Rectangle, Row, ScrollArea, Shape, Stack, Text,
    TextInput, Widget,
};

pub use placement::Justify::{SpaceAround, SpaceBetween, SpaceEvenly};
pub use placement::Size::Parent;
pub use window::{Window, close_window, open_window, window_size};
