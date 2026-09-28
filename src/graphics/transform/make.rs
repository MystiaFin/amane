use super::Transform;

impl Transform {
    pub const IDENTITY: Self = Self::from_scale(1.0, 1.0);

    pub const fn from_translate(tx: f32, ty: f32) -> Self {
        Self::from_row(1.0, 0.0, 0.0, 1.0, tx, ty)
    }

    // clockwise on screen, since y grows downward
    pub fn from_rotate(degrees: f32) -> Self {
        let (sin, cos) = degrees.to_radians().sin_cos();

        Self::from_row(cos, sin, -sin, cos, 0.0, 0.0)
    }
}
