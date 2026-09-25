mod column;
mod layout;
mod radius;
mod rectangle;
mod row;
mod size;
mod text;

use crate::graphics::{Rect, Renderer};

pub use column::Column;
pub use layout::{Direction, Layout};
pub use radius::Radius;
pub use rectangle::Rectangle;
pub use row::Row;
pub use size::Size;
pub use text::Text;

pub trait Widget {
    fn width(&self) -> Size;

    fn height(&self) -> Size;

    fn draw(&self, renderer: &mut Renderer, area: Rect);
}
