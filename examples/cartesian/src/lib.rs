#![no_std]

use generic_array_struct::generic_array_struct;

/// A `(x, y)` cartesian coordinate pair
#[generic_array_struct]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct CartesianInner<T> {
    /// x-coordinate
    pub x: T,

    /// y-coordinate
    pub y: T,
}

/// A `(x, y)` cartesian coordinate pair using `f64` values
pub type Cartesian = CartesianInner<f64>;

impl Cartesian {
    /// `(0.0, 0.0)`
    pub const ORIGIN: Self = Self([0.0; 2]);
}

impl Default for Cartesian {
    #[inline]
    fn default() -> Self {
        Self::ORIGIN
    }
}

#[generic_array_struct(pub)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct CartesianPub<T> {
    /// x-coordinate
    pub x: T,

    /// y-coordinate
    pub y: T,
}

pub mod inner {
    use super::*;

    #[generic_array_struct(pub(in crate::inner))]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[repr(transparent)]
    pub struct CartesianPubIn<T> {
        /// x-coordinate
        pub x: T,

        /// y-coordinate
        pub y: T,
    }

    #[generic_array_struct(pub(crate))]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[repr(transparent)]
    pub struct CartesianPubCrate<T> {
        /// x-coordinate
        pub x: T,

        /// y-coordinate
        pub y: T,
    }
}
