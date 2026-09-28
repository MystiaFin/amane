use crate::graphics::image::Bitmap;
use crate::graphics::{Cap, Color, Path, Rect, Transform};

pub enum Command {
    Fill {
        path: Path,
        transform: Transform,
        color: Color,
    },

    // a line along the path, centered on it
    Stroke {
        path: Path,
        transform: Transform,
        thickness: f32,
        color: Color,
        cap: Cap,
    },

    // the image is only shown inside the clip path
    Image {
        image: &'static Bitmap,
        transform: Transform,
        clip: Path,
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
    Layer {
        commands: Vec<Command>,
        opacity: f32,
    },

    // commands that only show inside the path
    Clip {
        path: Path,
        transform: Transform,
        commands: Vec<Command>,
    },
}
