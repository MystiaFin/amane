mod invert;
mod make;

/*
 * moves, scales and skews points:
 * x becomes sx * x + kx * y + tx, and y becomes ky * x + sy * y + ty
 */
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub(crate) sx: f32,
    pub(crate) ky: f32,
    pub(crate) kx: f32,
    pub(crate) sy: f32,
    pub(crate) tx: f32,
    pub(crate) ty: f32,
}

impl Transform {
    pub const fn from_row(sx: f32, ky: f32, kx: f32, sy: f32, tx: f32, ty: f32) -> Self {
        Self {
            sx,
            ky,
            kx,
            sy,
            tx,
            ty,
        }
    }

    pub const fn from_scale(sx: f32, sy: f32) -> Self {
        Self::from_row(sx, 0.0, 0.0, sy, 0.0, 0.0)
    }

    // the result does this transform first, then the other one
    pub fn post_concat(self, other: Transform) -> Self {
        Self {
            sx: other.sx * self.sx + other.kx * self.ky,
            ky: other.ky * self.sx + other.sy * self.ky,
            kx: other.sx * self.kx + other.kx * self.sy,
            sy: other.ky * self.kx + other.sy * self.sy,
            tx: other.sx * self.tx + other.kx * self.ty + other.tx,
            ty: other.ky * self.tx + other.sy * self.ty + other.ty,
        }
    }

    // the scale when the transform only moves and scales evenly, none when it rotates or stretches
    pub fn even_scale(self) -> Option<f32> {
        let even = (self.sx - self.sy).abs() < 0.0001;
        let straight = self.kx == 0.0 && self.ky == 0.0;

        if !even || !straight || self.sx <= 0.0 {
            return None;
        }

        Some(self.sx)
    }
}
