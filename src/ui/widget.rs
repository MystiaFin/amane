use crate::graphics::Renderer;

pub trait Widget {
    fn width(&self) -> f32;

    fn height(&self) -> f32;

    fn draw(&self, renderer: &mut Renderer, x: f32, y: f32);
}
