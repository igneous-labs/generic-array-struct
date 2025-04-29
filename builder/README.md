# generic-array-struct-builder

An attribute proc macro to create a builder struct of a [`generic-array-struct`](https://docs.rs/generic-array-struct) that at compile-time ensures that every field is set exactly once.

This crate must be used with structs that are `#[generic_array_struct]`.

## Example Usage

TODO: show example of stuff failing to compile because
- attempted to set more than once
- did not set all fields

## Implementation

```rust ignore
// TODO: remove ignore
use generic_array_struct::generic_array_struct;
use generic_array_struct_builder::generic_array_struct_builder;

#[generic_array_struct]
#[generic_array_struct_builder]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cartesian<T> {
    /// x-coordinate
    pub x: T,

    /// y-coordinate
    pub y: T,
}
```

expands to

```rust
use generic_array_struct::generic_array_struct;

#[generic_array_struct]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cartesian<T> {
    /// x-coordinate
    pub x: T,

    /// y-coordinate
    pub y: T,
}

// The const generic booleans track which fields have been set
#[repr(transparent)]
pub struct CartesianBuilder<T, const S0: bool, const S1: bool>(Cartesian<core::mem::MaybeUninit<T>>);

pub type NewCartesianBuilder<T> = CartesianBuilder<T, false, false>;

impl<T> NewCartesianBuilder<T> {
    const _UNINIT: core::mem::MaybeUninit<T> = core::mem::MaybeUninit::uninit();

    #[inline]
    pub const fn start() -> Self {
        Self(Cartesian([Self::_UNINIT; CARTESIAN_LEN]))
    }
}

// impl notes:
// - cannot use transmute() due to generic, cannot move out of struct due to Drop.
//   Hopefully rustc is able to optimize away all the 
//   transmute_copy() + core::mem::forget()s and use the same memory.
//   I cannot wait for array transmutes to be stabilized.

impl<T, const S1: bool> CartesianBuilder<T, false, S1> {
    #[inline]
    pub fn with_x(
        mut self,
        val: T,
    ) -> CartesianBuilder<T, true, S1> {
        // use raw array indices instead of mut references to preserve const
        self.0.0[CARTESIAN_IDX_X] = core::mem::MaybeUninit::new(val);
        unsafe {
            core::mem::transmute_copy::<_, _>(
                &core::mem::ManuallyDrop::new(self)
            )
        }
    }
}

impl<T, const S0: bool> CartesianBuilder<T, S0, false> {
    #[inline]
    pub fn with_y(
        mut self,
        val: T,
    ) -> CartesianBuilder<T, S0, true> {
        self.0.0[CARTESIAN_IDX_Y] = core::mem::MaybeUninit::new(val);
        unsafe {
            core::mem::transmute_copy::<_, _>(
                &core::mem::ManuallyDrop::new(self)
            )
        }
    }
}

impl<T> CartesianBuilder<T, true, true> {
    #[inline]
    pub fn build(self) -> Cartesian<T> {
        // if not `repr(transparent)`, must use `self.0` instead of `self`,
        // but we always enforce repr(transparent)
        unsafe {
            core::mem::transmute_copy::<_, _>(
                &core::mem::ManuallyDrop::new(self)
            )
        }
    }
}

/// This gets called if the Builder struct was dropped before `self.build()` was called
impl<T, const S0: bool, const S1: bool> Drop for CartesianBuilder<T, S0, S1> {
    fn drop(&mut self) {
        if S0 {
            unsafe {
                self.0.x_mut().assume_init_drop();
            }
        }
        if S1 {
            unsafe {
                self.0.y_mut().assume_init_drop();
            } 
        }
    }
}
```
