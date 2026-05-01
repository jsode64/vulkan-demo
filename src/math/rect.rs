use crate::Num;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Rect<T: Num> {
    pub x: T,

    pub y: T,

    pub w: T,

    pub h: T,
}

impl<T: Num> Rect<T> {
    /// Creates a rectangle with the given position and dimensions.
    #[inline]
    #[must_use]
    pub const fn new(x: T, y: T, w: T, h: T) -> Self {
        Self { x, y, w, h }
    }

    /// Centers the rectangle at the given coordinates.
    pub fn center_at(&mut self, x: T, y: T) {
        self.x = x - (self.w / T::TWO);
        self.y = y - (self.h / T::TWO);
    }
}
