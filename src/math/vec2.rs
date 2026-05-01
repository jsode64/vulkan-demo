use std::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign,
};

use crate::{Float, Num, Signed};

/// A 2-dimensional vector.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Vec2<T: Num = f32> {
    pub x: T,

    pub y: T,
}

impl<T: Num> Vec2<T> {
    /// A vector with an x and y of zero.
    pub const ZERO: Self = Vec2::new_v(T::ZERO);

    /// A vector with an x and y of one.
    pub const ONE: Self = Vec2::new_v(T::ONE);

    /// A vector pointing right.
    pub const RIGHT: Self = Vec2::new(T::ONE, T::ZERO);

    /// A vector pointing up.
    pub const UP: Self = Vec2::new(T::ZERO, T::ONE);

    /// Creates a vector with the given x and y factors.
    #[inline]
    #[must_use]
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    /// Creates a vector with the given x and y factor.
    #[inline]
    #[must_use]
    pub const fn new_v(v: T) -> Self {
        Self::new(v, v)
    }

    /// Returns the manhattan distance between the vectors.
    #[inline]
    #[must_use]
    pub fn dst_to_man(self, other: Self) -> T {
        self.x.safe_diff(other.x).abs() + self.y.safe_diff(other.y).abs()
    }

    /// Returns the distance between the vectors squared.
    #[inline]
    #[must_use]
    pub fn dst_to_sqr(self, other: Self) -> T {
        let dx = self.x - other.x;
        let dy = self.y - other.y;

        (dx * dx) + (dy * dy)
    }

    /// Returns the vector's magnitude squared.
    #[inline]
    #[must_use]
    pub fn mag_sqr(self) -> T {
        (self.x * self.x) + (self.y * self.y)
    }
}

impl<T: Signed> Vec2<T> {
    /// A vector pointing left.
    pub const LEFT: Self = Vec2::new(T::NEG_ONE, T::ZERO);

    /// A vector pointing down.
    pub const DOWN: Self = Vec2::new(T::ZERO, T::NEG_ONE);
}

impl<T: Float> Vec2<T> {
    /// Returns the distance between the vectors.
    #[inline]
    #[must_use]
    pub fn dst(self, other: Self) -> T {
        self.dst_to_sqr(other).sqrt()
    }

    /// Returns the vector's magnitude.
    #[inline]
    #[must_use]
    pub fn mag(self) -> T {
        self.mag_sqr().sqrt()
    }

    /// Returns the vector normalized or `None` if its magnitude it zero.
    #[inline]
    #[must_use]
    pub fn normalized(self) -> Option<Self> {
        let mag = self.mag();

        (mag != T::ZERO).then(|| self / mag)
    }
}

impl<T: Signed> Neg for Vec2<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}

macro_rules! impl_op_for_vec2 {
    ($op_trait:ident, $op_fn:ident) => {
        impl<T: Num> $op_trait<T> for Vec2<T> {
            type Output = Self;

            fn $op_fn(self, rhs: T) -> Self::Output {
                let x = self.x.$op_fn(rhs);
                let y = self.y.$op_fn(rhs);
                Self::new(x, y)
            }
        }

        impl<T: Num> $op_trait<Self> for Vec2<T> {
            type Output = Self;

            fn $op_fn(self, rhs: Self) -> Self {
                let x = self.x.$op_fn(rhs.x);
                let y = self.y.$op_fn(rhs.y);
                Self::new(x, y)
            }
        }
    };
}

impl_op_for_vec2!(Add, add);
impl_op_for_vec2!(Sub, sub);
impl_op_for_vec2!(Mul, mul);
impl_op_for_vec2!(Div, div);
impl_op_for_vec2!(Rem, rem);

macro_rules! impl_op_assign_for_vec2 {
    ($op_assign_trait:ident, $op_assign_fn:ident) => {
        impl<T: Num> $op_assign_trait<T> for Vec2<T> {
            fn $op_assign_fn(&mut self, rhs: T) {
                self.x.$op_assign_fn(rhs);
                self.y.$op_assign_fn(rhs);
            }
        }

        impl<T: Num> $op_assign_trait<Self> for Vec2<T> {
            fn $op_assign_fn(&mut self, rhs: Self) {
                self.x.$op_assign_fn(rhs.x);
                self.y.$op_assign_fn(rhs.y);
            }
        }
    };
}

impl_op_assign_for_vec2!(AddAssign, add_assign);
impl_op_assign_for_vec2!(SubAssign, sub_assign);
impl_op_assign_for_vec2!(MulAssign, mul_assign);
impl_op_assign_for_vec2!(DivAssign, div_assign);
impl_op_assign_for_vec2!(RemAssign, rem_assign);
