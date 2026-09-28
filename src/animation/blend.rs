use crate::Color;

// anything an animation can move between, `amount` goes from 0 at `from` to 1 at `to`
pub trait Blend: Copy + PartialEq + Send + Sync + 'static {
    fn blend(from: Self, to: Self, amount: f32) -> Self;
}

impl Blend for f32 {
    fn blend(from: Self, to: Self, amount: f32) -> Self {
        from + (to - from) * amount
    }
}

impl Blend for Color {
    fn blend(from: Self, to: Self, amount: f32) -> Self {
        let r = blend_channel(from.r, to.r, amount);
        let g = blend_channel(from.g, to.g, amount);
        let b = blend_channel(from.b, to.b, amount);
        let a = blend_channel(from.a, to.a, amount);

        Color::rgba(r, g, b, a)
    }
}

fn blend_channel(from: u8, to: u8, amount: f32) -> u8 {
    let from = f32::from(from);
    let to = f32::from(to);

    let blended = f32::blend(from, to, amount);

    blended.round() as u8
}
