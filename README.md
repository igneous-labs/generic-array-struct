# generic-array-struct

An attribute proc macro to convert structs with named fields of the same generic type into a single-array-field tuple struct with array-index-based accessor and mutator methods.

## Example Usage

```rust
use generic_array_struct::generic_array_struct;

#[generic_array_struct]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Cartesian<T> {
    pub x: T,
    pub y: T,
}
```

expands to

```rust
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Cartesian<T>(pub [T; CARTESIAN_LEN]);

impl<T> Cartesian<T> {
    pub const fn x(&self) -> &T {
        &self.0[CARTESIAN_IDX_X]
    }

    /// Returns the old field value
    pub const fn set_x(&mut self, val: T) -> T {
        core::mem::replace(&mut self.0[CARTESIAN_IDX_X], val)
    }

    pub fn with_x(mut self, val: T) -> Self {
        self.0[CARTESIAN_IDX_X] = val;
        self
    }

    pub const fn y(&self) -> &T {
        &self.0[CARTESIAN_IDX_Y]
    }

    /// Returns the old field value
    pub const fn set_y(&mut self, val: T) -> T {
        core::mem::replace(&mut self.0[CARTESIAN_IDX_Y], val)
    }

    pub fn with_y(mut self, val: T) -> Self {
        self.0[CARTESIAN_IDX_Y] = val;
        self
    }
}

impl<T: Copy> Cartesian<T> {
    pub const fn const_with_x(mut self, val: T) -> Self {
        self.0[CARTESIAN_IDX_X] = val;
        self
    }

    pub const fn const_with_y(mut self, val: T) -> Self {
        self.0[CARTESIAN_IDX_Y] = val;
        self
    }
}

pub const CARTESIAN_LEN: usize = 2;

pub const CARTESIAN_IDX_X: usize = 0;
pub const CARTESIAN_IDX_Y: usize = 1;
```

## Usage Notes

<div class="warning">

Because this attribute modifies the struct definition, it must be placed above any derive attributes

</div>

## MSRV

`rustc 1.83.0` (stabilization of `core::mem::replace()` in `const`)
