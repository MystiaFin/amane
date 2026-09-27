use crate::graphics::image::Bitmap;
use crate::graphics::{Color, Path, Transform};

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
    },

    // the image is only shown inside the clip path
    Image {
        image: &'static Bitmap,
        transform: Transform,
        clip: Path,
        clip_transform: Transform,
    },

    // blurs what the commands before it drew inside the path, amount is in logical pixels
    Blur {
        path: Path,
        transform: Transform,
        amount: f32,
    },

    // commands drawn on their own, then laid over the rest at an opacity
    Layer {
        commands: Vec<Command>,
        opacity: f32,
    },
}
