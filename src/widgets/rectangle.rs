mod child;
mod corners;
mod draw;
mod targets;
mod transform;

use std::path::PathBuf;

use crate::graphics::{Color, VALUE_ROWS};
use crate::input::Handlers;
use crate::{Align, Fill, Padding, Shadow, Size};

use super::Widget;

use corners::Rounding;

pub struct Rectangle {
    pub(crate) width: Size,
    pub(crate) height: Size,

    // how it looks
    pub(crate) fill: Fill,
    pub(crate) rounding: Rounding,
    pub(crate) border_thickness: f32,
    pub(crate) border_color: Color,
    pub(crate) shadow: Option<Shadow>,

    // what's drawn behind it is blurred this far, in logical pixels, 0 for none
    pub(crate) blur: f32,

    // 0 is invisible, 1 fully shown
    pub(crate) opacity: f32,

    // a fragment shader drawn over the fill, and the numbers it reads
    pub(crate) shader: Option<PathBuf>,
    pub(crate) shader_values: Vec<[f32; 4]>,

    // what sits inside it, and where
    pub(crate) child: Option<Box<dyn Widget>>,
    pub(crate) padding: Padding,
    pub(crate) child_horizontal: Align,
    pub(crate) child_vertical: Align,
    pub(crate) clip: bool,

    // moves the drawing around its center without changing the layout; rotation is in degrees
    pub(crate) rotation: f32,
    pub(crate) scale: f32,
    pub(crate) translate_x: f32,
    pub(crate) translate_y: f32,

    // clicks, hovers and scrolls
    pub(crate) handlers: Handlers,
}

pub struct RectangleNeedsWidth;

pub struct RectangleNeedsHeight {
    pub(crate) width: Size,
}

impl Rectangle {
    // hands out a builder that needs a width, then a height, so neither can be forgotten
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> RectangleNeedsWidth {
        RectangleNeedsWidth
    }

    // what every rectangle starts as, once it has a size
    fn from_size(width: Size, height: Size) -> Self {
        Self {
            width,
            height,

            fill: Fill::Color(Color::TRANSPARENT),
            rounding: Rounding::Square,
            border_thickness: 0.0,
            border_color: Color::TRANSPARENT,
            shadow: None,
            blur: 0.0,
            opacity: 1.0,
            shader: None,
            shader_values: Vec::new(),

            child: None,
            padding: Padding::default(),
            child_horizontal: Align::Start,
            child_vertical: Align::Start,
            clip: false,

            rotation: 0.0,
            scale: 1.0,
            translate_x: 0.0,
            translate_y: 0.0,

            handlers: Handlers::default(),
        }
    }

    pub fn fill(mut self, fill: impl Into<Fill>) -> Self {
        self.fill = fill.into();

        self
    }

    pub fn border(mut self, thickness: f32, color: impl Into<Color>) -> Self {
        self.border_thickness = thickness;
        self.border_color = color.into();

        self
    }

    // blurs what's drawn behind the rectangle, in logical pixels
    pub fn blur(mut self, amount: f32) -> Self {
        self.blur = amount;

        self
    }

    // 0 is invisible, 1 fully shown
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;

        self
    }

    pub fn shadow(mut self, shadow: Shadow) -> Self {
        self.shadow = Some(shadow);

        self
    }

    /*
     * a wgsl, or .glsl and .frag, fragment shader drawn over the fill;
     * it gets uv, size and time, see examples/shader.rs
     */
    pub fn shader(mut self, path: impl Into<PathBuf>) -> Self {
        self.shader = Some(path.into());

        self
    }

    /*
     * numbers the shader reads as `values`, up to 16 rows of four,
     * like the rectangles a shader should draw; rows left out are 0
     */
    pub fn shader_values(mut self, values: Vec<[f32; 4]>) -> Self {
        assert!(
            values.len() <= VALUE_ROWS,
            "a shader takes at most {VALUE_ROWS} rows of values"
        );

        self.shader_values = values;

        self
    }

    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(child));

        self
    }

    // space between the edges and the child, one number for every side or a Padding for each
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();

        self
    }

    // where the child sits inside the padding, left to right and then top to bottom
    pub fn align_child(mut self, horizontal: impl Into<Align>, vertical: impl Into<Align>) -> Self {
        self.child_horizontal = horizontal.into();
        self.child_vertical = vertical.into();

        self
    }

    // the child only shows inside the rectangle, following its rounded corners
    pub fn clip(mut self) -> Self {
        self.clip = true;

        self
    }
}

impl RectangleNeedsWidth {
    pub fn width(self, width: impl Into<Size>) -> RectangleNeedsHeight {
        RectangleNeedsHeight {
            width: width.into(),
        }
    }
}

impl RectangleNeedsHeight {
    pub fn height(self, height: impl Into<Size>) -> Rectangle {
        Rectangle::from_size(self.width, height.into())
    }
}
