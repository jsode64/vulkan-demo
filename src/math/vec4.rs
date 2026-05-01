/// A 3-dimensional vector.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec4<T = f32> {
    pub x: T,

    pub y: T,

    pub z: T,

    pub w: T,
}

impl<T> Vec4<T> {
    /// Creates a vector with the given x y and z factors.
    #[inline]
    #[must_use]
    pub const fn new(x: T, y: T, z: T, w: T) -> Self {
        Self { x, y, z, w }
    }
}
