#![no_std]

use generic_array_struct::generic_array_struct;

/// A RGB color triple
#[generic_array_struct(builder pub)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Rgb<T> {
    /// red
    pub r: T,

    /// green
    pub g: T,

    /// blue
    pub b: T,
}

pub type RgbU8 = Rgb<u8>;

pub fn white() -> RgbU8 {
    NewRgbBuilder::start()
        .with_r(255)
        .with_g(255)
        .with_b(255)
        .build()
}
