mod column;
mod layout;
mod rectangle;
mod row;

use crate::graphics::Renderer;

pub use column::Column;
pub use layout::{Direction, Layout};
pub use rectangle::Rectangle;
pub use row::Row;

pub trait Widget {
    fn width(&self) -> f32;

    fn height(&self) -> f32;

    fn draw(&self, renderer: &mut Renderer, x: f32, y: f32);
}
