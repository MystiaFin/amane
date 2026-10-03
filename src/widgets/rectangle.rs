mod child;
mod draw;
mod input;
mod transform;

use std::path::PathBuf;

use crate::graphics::{Color, VALUE_ROWS};
use crate::input::Handlers;
use crate::{Align, Fill, Padding, Radius, Shadow, Size};

use super::Widget;

pub struct Rectangle {
    pub(crate) width: Size,
    pub(crate) height: Size,
    pub(crate) fill: Fill,
    pub(crate) radius: Radius,
    pub(crate) border_thickness: f32,
    pub(crate) border_color: Color,
    pub(crate) blur: f32,
    pub(crate) opacity: f32,
    pub(crate) shadow: Option<Shadow>,
    pub(crate) shader: Option<PathBuf>,
    pub(crate) shader_values: Vec<[f32; 4]>,
    pub(crate) child: Option<Box<dyn Widget>>,
    pub(crate) clip: bool,
    pub(crate) rotation: f32,
    pub(crate) scale: f32,
    pub(crate) translate_x: f32,
    pub(crate) translate_y: f32,
    pub(crate) padding: Padding,
    pub(crate) child_horizontal: Align,
    pub(crate) child_vertical: Align,
    pub(crate) handlers: Handlers,
}

pub struct NeedsWidth;

pub struct NeedsHeight {
    pub(crate) width: Size,
}

impl Rectangle {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> NeedsWidth {
        NeedsWidth
    }

    pub fn fill(mut self, fill: impl Into<Fill>) -> Self {
        self.fill = fill.into();

        self
    }

    pub fn radius(mut self, radius: impl Into<Radius>) -> Self {
        self.radius = radius.into();

        self
    }

    pub fn border(mut self, thickness: f32, color: impl Into<Color>) -> Self {
        self.border_thickness = thickness;
        self.border_color = color.into();

        self
    }

    pub fn blur(mut self, amount: f32) -> Self {
        self.blur = amount;

        self
    }

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
    pub fn align_child(
        mut self,
        horizontal: impl Into<Align>,
        vertical: impl Into<Align>,
    ) -> Self {
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

impl NeedsWidth {
    pub fn width(self, width: impl Into<Size>) -> NeedsHeight {
        NeedsHeight {
            width: width.into(),
        }
    }
}

impl NeedsHeight {
    pub fn height(self, height: impl Into<Size>) -> Rectangle {
        Rectangle {
            width: self.width,
            height: height.into(),
            radius: Radius::Fixed(0.0),
            fill: Fill::Color(Color::TRANSPARENT),
            border_thickness: 0.0,
            border_color: Color::TRANSPARENT,
            blur: 0.0,
            opacity: 1.0,
            shadow: None,
            shader: None,
            shader_values: Vec::new(),
            child: None,
            clip: false,
            rotation: 0.0,
            scale: 1.0,
            translate_x: 0.0,
            translate_y: 0.0,
            padding: Padding::default(),
            child_horizontal: Align::Start,
            child_vertical: Align::Start,
            handlers: Handlers::default(),
        }
    }
}
