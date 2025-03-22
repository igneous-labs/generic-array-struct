# generic-array-struct

An attribute proc macro to convert structs with named fields of the same generic type into a single-array-field tuple struct with array-index-based accessor and mutator methods.

## MSRV

`rustc 1.83.0` (stabilization of [`core::mem::replace()`](`core::mem::replace()`) in `const`)

## Example Usage

```rust
use generic_array_struct::generic_array_struct;

#[generic_array_struct]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Cartesian<T> {
    /// x-coordinate
    pub x: T,

    /// y-coordinate
    pub y: T,
}
```

expands to

```rust
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Cartesian<T>(pub [T; CARTESIAN_LEN]);

impl<T> Cartesian<T> {
    /// x-coordinate
    #[inline]
    pub const fn x(&self) -> &T {
        &self.0[CARTESIAN_IDX_X]
    }

    /// Returns the old field value
    #[inline]
    pub const fn set_x(&mut self, val: T) -> T {
        core::mem::replace(&mut self.0[CARTESIAN_IDX_X], val)
    }

    #[inline]
    pub fn with_x(mut self, val: T) -> Self {
        self.0[CARTESIAN_IDX_X] = val;
        self
    }

    /// y-coordinate
    #[inline]
    pub const fn y(&self) -> &T {
        &self.0[CARTESIAN_IDX_Y]
    }

    /// Returns the old field value
    #[inline]
    pub const fn set_y(&mut self, val: T) -> T {
        core::mem::replace(&mut self.0[CARTESIAN_IDX_Y], val)
    }

    #[inline]
    pub fn with_y(mut self, val: T) -> Self {
        self.0[CARTESIAN_IDX_Y] = val;
        self
    }
}

impl<T: Copy> Cartesian<T> {
    #[inline]
    pub const fn const_with_x(mut self, val: T) -> Self {
        self.0[CARTESIAN_IDX_X] = val;
        self
    }

    #[inline]
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

### Declaration Order

Because this attribute modifies the struct definition, it must be placed above any derive attributes or attributes that use the struct definition

#### WRONG ❌

```rust,compile_fail,E0609
use generic_array_struct::generic_array_struct;

// Fails to compile because #[generic_array_struct] is below #[derive] attribute
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[generic_array_struct]
pub struct Cartesian<D> {
    pub x: D,
    pub y: D,
}
```

#### RIGHT ✅

```rust
use generic_array_struct::generic_array_struct;

#[generic_array_struct]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cartesian<D> {
    pub x: D,
    pub y: D,
}
```

### Field Visibility

All methods have the same visibility as that of the originally declared field in the struct.

```rust,compile_fail,E0624
mod private {
    use generic_array_struct::generic_array_struct;

    #[generic_array_struct]
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Cartesian<T> {
        // Note: fields are private
        x: T,
        y: T,
    }
}

use private::Cartesian;

// fails to compile because [`Cartesian::const_with_x`] is private
const ONE_COMMA_ZERO: Cartesian<f64> = Cartesian([0.0; 2]).const_with_x(1.0);
```
