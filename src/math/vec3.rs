/// A 3-dimensional vector.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3<T = f32> {
    pub x: T,

    pub y: T,

    pub z: T,
}

impl<T> Vec3<T> {
    /// Creates a vector with the given x y and z factors.
    #[inline]
    #[must_use]
    pub const fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }
}
