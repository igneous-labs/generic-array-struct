#![cfg_attr(not(test), no_std)]

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

impl RgbU8 {
    pub fn white() -> Self {
        NewRgbBuilder::start()
            .with_r(255)
            .with_g(255)
            .with_b(255)
            .build()
    }

    pub fn red() -> Self {
        NewRgbBuilder::start()
            .with_r(255)
            .with_g(0)
            .with_b(0)
            .build()
    }

    pub fn green() -> Self {
        NewRgbBuilder::start()
            .with_r(0)
            .with_g(255)
            .with_b(0)
            .build()
    }

    pub fn blue() -> Self {
        NewRgbBuilder::start()
            .with_r(0)
            .with_g(0)
            .with_b(255)
            .build()
    }
}

pub const BLACK: RgbU8 = NewRgbBuilder::start().with_b(0).with_g(0).with_r(0).build();

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;

    #[test]
    fn assert_const_colors() {
        assert_eq!(RgbU8::white().0, [255, 255, 255]);
        assert_eq!(RgbU8::red().0, [255, 0, 0]);
        assert_eq!(RgbU8::green().0, [0, 255, 0]);
        assert_eq!(RgbU8::blue().0, [0, 0, 255]);
    }

    #[test]
    fn mem_safety_forget_builder() {
        let a = NewRgbBuilder::start()
            .with_r(vec![1])
            .with_g(vec![2])
            .with_b(vec![3])
            .build();
        eprintln!("{a:#?}");
        // a gets dropped here. If Builder not mem::forgotten, double free will occur
        // and segfault
    }

    #[test]
    fn mem_safety_drop_builder() {
        let r = Rc::new(1);

        let a = NewRgbBuilder::start().with_r(r.clone()).with_g(r);
        eprintln!("{:#?}", a.0);
        // partially initialized a gets dropped here.
        // Make sure Builder `Drop` impl doesnt
        // attempt to drop uninitialized memory.
        // If so, this will segfault for attempting to decrement nonexistent Rc.
    }

    // TODO: need to find a (easy) way to test for memory leaks
}
