mod canvas;
mod column;
mod layout;
mod rectangle;
mod row;
mod scroll_area;
mod shape;
mod stack;
mod text;
mod text_input;

use crate::Size;
use crate::graphics::{Area, Renderer};
use crate::input::Target;

pub use canvas::Canvas;
pub use column::Column;
pub use layout::{Direction, Layout};
pub use rectangle::Rectangle;
pub use row::Row;
pub use scroll_area::ScrollArea;
pub use shape::{Arc, Circle, Line, Path, Shape};
pub use stack::Stack;
pub use text::Text;
pub use text_input::TextInput;

pub trait Widget {
    fn width(&self) -> Size;

    fn height(&self) -> Size;

    fn draw(&self, renderer: &mut Renderer, area: Area);

    // widgets that react to the pointer add their area, containers pass theirs on to children
    fn collect_targets(&self, _: Area, _: &mut Vec<Target>) {}
}
