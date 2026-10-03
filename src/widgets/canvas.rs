use crate::graphics::{Area, Renderer};
use crate::{Color, Shape, Size, Widget};

// draws its shapes in its own coordinates, with 0,0 at its top left corner
pub struct Canvas {
    pub(crate) width: Size,
    pub(crate) height: Size,
    pub(crate) shapes: Vec<Box<dyn Shape>>,
}

pub struct CanvasNeedsWidth;

pub struct CanvasNeedsHeight {
    pub(crate) width: Size,
}

impl Canvas {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> CanvasNeedsWidth {
        CanvasNeedsWidth
    }

    // later shapes draw over earlier ones
    pub fn shapes(mut self, shapes: Vec<Box<dyn Shape>>) -> Self {
        self.shapes = shapes;

        self
    }
}

impl CanvasNeedsWidth {
    pub fn width(self, width: impl Into<Size>) -> CanvasNeedsHeight {
        CanvasNeedsHeight {
            width: width.into(),
        }
    }
}

impl CanvasNeedsHeight {
    pub fn height(self, height: impl Into<Size>) -> Canvas {
        Canvas {
            width: self.width,
            height: height.into(),
            shapes: Vec::new(),
        }
    }
}

impl Widget for Canvas {
    fn width(&self) -> Size {
        self.width
    }

    fn height(&self) -> Size {
        self.height
    }

    fn draw(&self, renderer: &mut Renderer, area: Area) {
        for shape in &self.shapes {
            paint(shape.as_ref(), renderer, area);
        }
    }
}

fn paint(shape: &dyn Shape, renderer: &mut Renderer, area: Area) {
    let Some(path) = shape.trace(area.width, area.height) else {
        return;
    };

    let style = shape.style();

    // fill and line are faded together, so the line doesn't show the fill through it
    let mut group = renderer.group();

    // shapes start without a fill, and drawing an invisible one would still cost the gpu
    if style.fill != Color::TRANSPARENT {
        group.fill_path(&path, area, style.fill);
    }

    // a zero width line would still draw as a hairline
    if style.stroke_thickness > 0.0 {
        group.stroke_path(
            &path,
            area,
            style.stroke_thickness,
            style.stroke_color,
            style.cap,
        );
    }

    renderer.blend(group, style.opacity);
}
