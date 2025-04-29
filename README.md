# generic-array-struct

An attribute proc macro to convert structs with named fields of the same generic type into a single-array-field tuple struct with array-index-based accessor and mutator methods.

## MSRV

`rustc 1.64.0` (`const fn`, `workspace = true`)

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
pub struct Cartesian<T>([T; CARTESIAN_LEN]);

impl<T> Cartesian<T> {
    /// x-coordinate
    #[inline]
    pub const fn x(&self) -> &T {
        &self.0[CARTESIAN_IDX_X]
    }

    #[inline]
    pub fn x_mut(&mut self) -> &mut T {
        &mut self.0[CARTESIAN_IDX_X]
    }

    /// Returns the old field value
    #[inline]
    pub fn set_x(&mut self, val: T) -> T {
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

    #[inline]
    pub fn y_mut(&mut self) -> &mut T {
        &mut self.0[CARTESIAN_IDX_Y]
    }

    /// Returns the old field value
    #[inline]
    pub fn set_y(&mut self, val: T) -> T {
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

// consts are exported with prefix instead of as associated consts
// so that we dont need turbofish e.g. `Cartesian::<f32>::IDX_X`

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

### Attribute args

The attribute can be further customized by the following space-separated positional args.

#### Builder Arg

An optional `builder` positionl arg controls whether to generate a builder struct that at compile-time ensures that every field is set exactly once.

```rust
use generic_array_struct::generic_array_struct;

#[generic_array_struct(builder pub)]
pub struct Cartesian<Z> {
    pub x: Z,
    pub y: Z,
}
```

expands to

```rust
use generic_array_struct::generic_array_struct;

#[generic_array_struct(pub)]
pub struct Cartesian<Z> {
    pub x: Z,
    pub y: Z,
}

// The const generic booleans track which fields have been set
#[repr(transparent)]
pub struct CartesianBuilder<Z, const S0: bool, const S1: bool>(Cartesian<core::mem::MaybeUninit<Z>>);

pub type NewCartesianBuilder<Z> = CartesianBuilder<Z, false, false>;

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

impl<Z, const S1: bool> CartesianBuilder<Z, false, S1> {
    #[inline]
    pub fn with_x(
        mut self,
        val: Z,
    ) -> CartesianBuilder<Z, true, S1> {
        // use raw array indices instead of mut references to preserve const
        self.0.0[CARTESIAN_IDX_X] = core::mem::MaybeUninit::new(val);
        unsafe {
            core::mem::transmute_copy::<_, _>(
                &core::mem::ManuallyDrop::new(self)
            )
        }
    }
}

impl<Z, const S0: bool> CartesianBuilder<Z, S0, false> {
    #[inline]
    pub fn with_y(
        mut self,
        val: Z,
    ) -> CartesianBuilder<Z, S0, true> {
        self.0.0[CARTESIAN_IDX_Y] = core::mem::MaybeUninit::new(val);
        unsafe {
            core::mem::transmute_copy::<_, _>(
                &core::mem::ManuallyDrop::new(self)
            )
        }
    }
}

impl<Z> CartesianBuilder<Z, true, true> {
    #[inline]
    pub fn build(self) -> Cartesian<Z> {
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
impl<Z, const S0: bool, const S1: bool> Drop for CartesianBuilder<Z, S0, S1> {
    fn drop(&mut self) {
        if S0 {
            unsafe {
                self.0.0[CARTESIAN_IDX_X].assume_init_drop();
            }
        }
        if S1 {
            unsafe {
                self.0.0[CARTESIAN_IDX_Y].assume_init_drop();
            } 
        }
    }
}
```

TODO: show example of stuff failing to compile because
- attempted to set more than once
- did not set all fields


#### `.0` Visibility Attribute Arg

The attribute's second position arg is a [`syn::Visibility`](`syn::Visibility`) that controls the visibility of the resulting `.0` array field. 

```rust
use generic_array_struct::generic_array_struct;

#[generic_array_struct]
pub struct Cartesian<T> {
    pub x: T,
    pub y: T,
}
```

generates

```rust
pub struct Cartesian<T>([T; 2]);
```

while

```rust
use generic_array_struct::generic_array_struct;

#[generic_array_struct(pub(crate))]
pub struct Cartesian<T> {
    pub x: T,
    pub y: T,
}
```

generates

```rust
pub struct Cartesian<T>(pub(crate) [T; 2]);
```
