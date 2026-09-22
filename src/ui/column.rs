use crate::graphics::Renderer;

use super::Widget;

pub struct Column {
    children: Vec<Box<dyn Widget>>,
}

impl Column {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn push<W>(&mut self, child: W)
    where
        W: Widget + 'static,
    {
        self.children.push(Box::new(child));
    }
}

impl Widget for Column {
    fn width(&self) -> f32 {
        self.children
            .iter()
            .map(|child| child.width())
            .fold(0.0, f32::max)
    }

    fn height(&self) -> f32 {
        self.children.iter().map(|child| child.height()).sum()
    }

    fn draw(&self, renderer: &mut Renderer, x: f32, y: f32) {
        let mut current_y = y;

        for child in &self.children {
            child.draw(renderer, x, current_y);

            current_y += child.height();
        }
    }
}
