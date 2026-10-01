use std::any::TypeId;
use std::collections::HashSet;

use crate::Widget;
use crate::changes;
use crate::graphics::{Rect, Renderer};
use crate::input::Target;

// one window's frame: what the gpu draws, and where the widgets take the pointer
pub struct Frame {
    pub renderer: Renderer,
    pub targets: Vec<Target>,

    // services read while drawing, on top of the ones the view read
    pub reads: HashSet<TypeId>,
}

/*
 * runs a window's view at the window's size; the services it read come
 * back with it, so a later change to one of them draws the window again
 */
pub fn run_view<T>(view: impl FnOnce() -> T, width: u32, height: u32) -> (T, HashSet<TypeId>) {
    // anything read before belongs to another window
    changes::take_read();

    crate::window::set_size(width as f32, height as f32);

    // the view runs again on every redraw, so it shows the services as they are now
    let content = view();

    let reads = changes::take_read();

    (content, reads)
}

// lays the root out in the window, draws it, and collects where it reacts to the pointer
pub fn build(root: &dyn Widget, width: u32, height: u32, scale: f32) -> Frame {
    let area = root_area(root, width, height);

    let mut renderer = Renderer::new(scale);

    root.draw(&mut renderer, area);

    let reads = changes::take_read();

    let mut targets = Vec::new();

    root.collect_targets(area, &mut targets);

    Frame {
        renderer,
        targets,
        reads,
    }
}

// the root at its own size, from the window's top left corner
fn root_area(root: &dyn Widget, width: u32, height: u32) -> Rect {
    let root_width = root.width().resolve(width as f32);
    let root_height = root.height().resolve(height as f32);

    Rect::new(0.0, 0.0, root_width, root_height)
}
