mod canvas;
mod column;
mod fill;
mod fit;
mod image;
mod layout;
mod mask;
mod radius;
mod rectangle;
mod row;
mod shadow;
mod shape;
mod size;
mod text;

use crate::graphics::{Rect, Renderer};
use crate::input::Target;

pub use canvas::Canvas;
pub use column::Column;
pub use fill::Fill;
pub use image::Image;
pub use layout::{Direction, Layout};
pub use mask::Mask;
pub use radius::Radius;
pub use rectangle::Rectangle;
pub use row::Row;
pub use shadow::Shadow;
pub use shape::{Arc, Circle, Line, Path, Shape};
pub use size::Size;
pub use text::Text;

pub trait Widget {
    fn width(&self) -> Size;

    fn height(&self) -> Size;

    fn draw(&self, renderer: &mut Renderer, area: Rect);

    // widgets that react to the pointer add their area, containers pass theirs on to children
    fn collect_targets(&self, _: Rect, _: &mut Vec<Target>) {}
}
