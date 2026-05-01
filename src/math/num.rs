use std::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign,
};

/// A number (signed/unsigned integer or float).
pub trait Num:
    Clone
    + Copy
    + Sized
    + PartialEq
    + PartialOrd
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + SubAssign
    + Mul<Output = Self>
    + MulAssign
    + Div<Output = Self>
    + DivAssign
    + Rem<Output = Self>
    + RemAssign
{
    /// The type's 0 value.
    const ZERO: Self;

    /// The type's 1 value.
    const ONE: Self;

    /// The type's 2 value.
    const TWO: Self;

    /// Returns the value's absolute value.
    #[must_use]
    fn abs(self) -> Self;

    /// Returns the difference between the two values safely.
    ///
    /// For unsigned types, returns `self.abs_diff(other)` to prevent overflow.
    #[must_use]
    fn safe_diff(self, other: Self) -> Self;
}

/// A signed number.
pub trait Signed: Num + Neg<Output = Self> {
    /// The type's -1 value.
    const NEG_ONE: Self;
}

pub trait Float: Num {
    /// The type's π value.
    const PI: Self;

    /// Returns the value's square root.
    #[must_use]
    fn sqrt(self) -> Self;
}

macro_rules! impl_num {
    ($($t:ty)*; $abs:expr, $safe_diff:expr $(,)?) => {
        $(
            impl Num for $t {
                const ZERO: Self = 0 as Self;

                const ONE: Self = 1 as Self;

                const TWO: Self = 2 as Self;

                #[inline(always)]
                fn abs(self) -> Self {
                    $abs(self)
                }

                #[inline(always)]
                fn safe_diff(self, other: Self) -> Self {
                    $safe_diff(self, other)
                }
            }
        )*
    };
}

impl_num!(
    i8 i16 i32 i64 i128 isize f32 f64;
    Self::abs, Sub::sub
);

impl_num!(
    u8 u16 u32 u64 u128 usize;
    |n| n, Self::abs_diff,
);

macro_rules! impl_signed {
    ($($t:ty)*) => {
        $(
            impl Signed for $t {
                const NEG_ONE: Self = -1 as Self;
            }
        )*
    };
}

impl_signed!(i8 i16 i32 i64 i128 isize f32 f64);

macro_rules! impl_float {
    ($($t:ty)*) => {
        $(
            impl Float for $t {
                const PI: Self = std::f64::consts::PI as Self;

                #[inline(always)]
                fn sqrt(self) -> Self {
                    self.sqrt()
                }
            }
        )*
    };
}

impl_float!(f32 f64);
