use crate::graphics::image::Bitmap;
use std::path::PathBuf;
use std::sync::Arc;

use ttf_parser::Face;

use crate::graphics::{Cap, Color, Gradient, Path, Rect, Transform};

pub enum Command {
    // one letter as a picture, at a whole device pixel; x and y are where its baseline starts
    Glyph {
        face: &'static Face<'static>,
        id: u16,
        size: f32,
        color: Color,
        x: f32,
        y: f32,
    },

    // a rounded rectangle, kept as one so the gpu can draw it without tracing its outline
    Rectangle {
        rect: Rect,
        radius: f32,
        transform: Transform,
        color: Color,
    },

    // a line along the inside edge of a rounded rectangle
    Border {
        rect: Rect,
        radius: f32,
        thickness: f32,
        transform: Transform,
        color: Color,
    },

    Fill {
        path: Path,
        transform: Transform,
        color: Color,
    },

    // the rect says where the gradient starts and ends
    Gradient {
        path: Path,
        rect: Rect,
        transform: Transform,
        gradient: Gradient,
    },

    // a line along the path, centered on it
    Stroke {
        path: Path,
        transform: Transform,
        thickness: f32,
        color: Color,
        cap: Cap,
    },

    // the image is only shown inside the clip path, which traces the rounded rectangle
    Image {
        image: Arc<Bitmap>,
        transform: Transform,
        clip: Path,
        rect: Rect,
        radius: f32,
        clip_transform: Transform,
    },

    // a soft copy of the rounded rectangle, blur is in logical pixels
    Shadow {
        rect: Rect,
        radius: f32,
        transform: Transform,
        color: Color,
        blur: f32,
    },

    // the color inside the clip path, fading out toward the rounded hole
    InnerShadow {
        clip: Path,
        hole: Rect,
        radius: f32,
        transform: Transform,
        color: Color,
        blur: f32,
    },

    // a custom shader run over the rect, trimmed to the path when it has one; values are passed to it
    Shader {
        shader: PathBuf,
        values: Vec<[f32; 4]>,
        rect: Rect,
        path: Option<Path>,
        transform: Transform,
    },

    // blurs what the commands before it drew inside the path, amount is in logical pixels
    Blur {
        path: Path,
        transform: Transform,
        amount: f32,
    },

    // erases the inside of the path from what the commands before it drew, strength 1 erases fully
    Cut {
        path: Path,
        transform: Transform,
        strength: f32,
    },

    // commands drawn on their own, then laid over the rest at an opacity
    Group {
        commands: Vec<Command>,
        opacity: f32,
    },

    // commands that only show inside the path, which traces the rounded rectangle
    Clip {
        path: Path,
        rect: Rect,
        radius: f32,
        transform: Transform,
        commands: Vec<Command>,
    },
}
