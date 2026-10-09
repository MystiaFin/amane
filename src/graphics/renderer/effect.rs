use std::path::Path;

use crate::graphics::{Area, Color, Corners, Renderer};

use super::Command;

// how many rows of four numbers a shader can be handed as values
pub const VALUE_ROWS: usize = 16;

impl Renderer {
    pub fn shadow(&mut self, area: Area, radius: Corners, color: Color, blur: f32) {
        self.commands.push(Command::Shadow {
            area,
            radius,
            transform: self.transform,
            color,
            blur,
        });
    }

    // the shadow fills the rectangle except for a soft hole, so only a rim is left
    pub fn inner_shadow(
        &mut self,
        area: Area,
        hole: Area,
        radius: Corners,
        color: Color,
        blur: f32,
    ) {
        let Some(clip) = area.trace(radius) else {
            return;
        };

        self.commands.push(Command::InnerShadow {
            clip,
            hole,
            radius,
            transform: self.transform,
            color,
            blur,
        });
    }

    pub fn blur(&mut self, area: Area, radius: Corners, amount: f32) {
        // most rectangles ask for no blur, so they skip copying pixels
        if amount == 0.0 {
            return;
        }

        let Some(path) = area.trace(radius) else {
            return;
        };

        self.commands.push(Command::Blur {
            path,
            transform: self.transform,
            amount,
        });
    }

    pub fn cut(&mut self, area: Area, radius: Corners, strength: f32) {
        if area.width <= 0.0 || area.height <= 0.0 {
            return;
        }

        self.commands.push(Command::Cut {
            area,
            radius,
            transform: self.transform,
            strength,
        });
    }

    // runs the shader file over the rounded rectangle, handing it the values
    pub fn shader(&mut self, area: Area, shader: &Path, radius: Corners, values: &[[f32; 4]]) {
        let Some(outline) = area.trace(radius) else {
            return;
        };

        // the shader only covers the area, so square corners need no trimming
        let path = if radius.largest() > 0.0 {
            Some(outline)
        } else {
            None
        };

        self.commands.push(Command::Shader {
            shader: shader.to_path_buf(),
            values: values.to_vec(),
            area,
            path,
            transform: self.transform,
        });
    }
}
