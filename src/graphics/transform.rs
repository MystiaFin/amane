/*
 * moves, scales and skews points:
 * x becomes sx * x + kx * y + tx, and y becomes ky * x + sy * y + ty
 */
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub sx: f32,
    pub ky: f32,
    pub kx: f32,
    pub sy: f32,
    pub tx: f32,
    pub ty: f32,
}

impl Transform {
    pub const IDENTITY: Self = Self::from_scale(1.0, 1.0);

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

    pub const fn from_translate(tx: f32, ty: f32) -> Self {
        Self::from_row(1.0, 0.0, 0.0, 1.0, tx, ty)
    }

    // clockwise on screen, since y grows downward
    pub fn from_rotate(degrees: f32) -> Self {
        let (sin, cos) = degrees.to_radians().sin_cos();

        Self::from_row(cos, sin, -sin, cos, 0.0, 0.0)
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

    // none when the transform squashes everything flat, like a scale of 0
    pub fn invert(self) -> Option<Self> {
        let determinant = self.sx * self.sy - self.kx * self.ky;

        if determinant == 0.0 {
            return None;
        }

        let sx = self.sy / determinant;
        let ky = -self.ky / determinant;
        let kx = -self.kx / determinant;
        let sy = self.sx / determinant;

        let tx = (self.kx * self.ty - self.sy * self.tx) / determinant;
        let ty = (self.ky * self.tx - self.sx * self.ty) / determinant;

        Some(Self::from_row(sx, ky, kx, sy, tx, ty))
    }

    pub fn map(self, x: f32, y: f32) -> (f32, f32) {
        let mapped_x = self.sx * x + self.kx * y + self.tx;
        let mapped_y = self.ky * x + self.sy * y + self.ty;

        (mapped_x, mapped_y)
    }
}
