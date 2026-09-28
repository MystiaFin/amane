mod align;
mod canvas;
mod center;
mod column;
mod end;
mod fill;
mod fit;
mod image;
mod justify;
mod layout;
mod mask;
mod padding;
mod radius;
mod rectangle;
mod row;
mod scroll_area;
mod shadow;
mod shape;
mod size;
mod start;
mod text;
mod text_input;

use crate::graphics::{Rect, Renderer};
use crate::input::Target;

pub use align::Align;
pub use canvas::Canvas;
pub use center::Center;
pub use column::Column;
pub use end::End;
pub use fill::Fill;
pub use image::Image;
pub use justify::Justify;
pub use layout::{Direction, Layout};
pub use mask::Mask;
pub use padding::Padding;
pub use radius::Radius;
pub use rectangle::Rectangle;
pub use row::Row;
pub use scroll_area::ScrollArea;
pub use shadow::Shadow;
pub use shape::{Arc, Circle, Line, Path, Shape};
pub use size::Size;
pub use start::Start;
pub use text::Text;
pub use text_input::TextInput;

pub trait Widget {
    fn width(&self) -> Size;

    fn height(&self) -> Size;

    fn draw(&self, renderer: &mut Renderer, area: Rect);

    // widgets that react to the pointer add their area, containers pass theirs on to children
    fn collect_targets(&self, _: Rect, _: &mut Vec<Target>) {}
}
