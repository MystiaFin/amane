use super::Transform;

impl Transform {
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
