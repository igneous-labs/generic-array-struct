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
pub struct Cartesian<T>([T; CARTESIAN_LEN]);

impl<T> Cartesian<T> {
    /// x-coordinate
    #[inline]
    pub const fn x(&self) -> &T {
        &self.0[CARTESIAN_IDX_X]
    }

    #[inline]
    pub const fn x_mut(&mut self) -> &mut T {
        &mut self.0[CARTESIAN_IDX_X]
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

    #[inline]
    pub const fn y_mut(&mut self) -> &mut T {
        &mut self.0[CARTESIAN_IDX_Y]
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


impl<T> Cartesian<T> {
    pub const LEN: usize = 2;

    pub const IDX_X: usize = 0;
    pub const IDX_Y: usize = 1;
}

// consts are also exported with prefix (not just as associated consts)
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

#### `destr` Arg

An optional `destr` prefix arg controls whether to output the original struct definition as a separate struct for destructuring.

```rust
use generic_array_struct::generic_array_struct;

#[generic_array_struct(destr pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Cartesian<Z> {
    pub x: Z,
    pub y: Z,
}
```

expands to

```rust
use generic_array_struct::generic_array_struct;

#[generic_array_struct(pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Cartesian<Z> {
    pub x: Z,
    pub y: Z,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CartesianDestr<Z> {
    pub x: Z,
    pub y: Z,
}

impl<T> Cartesian<T> {
    #[inline]
    pub fn from_destr(CartesianDestr { x, y, }: CartesianDestr<T>) -> Self {
        Self([x, y,])
    }

    #[inline]
    pub fn into_destr(self) -> CartesianDestr<T> {
        let Self([x, y,]) = self;
        CartesianDestr { x, y, }
    }
}

impl<T: Copy> Cartesian<T> {
    #[inline]
    pub const fn const_from_destr(CartesianDestr { x, y, }: CartesianDestr<T>) -> Self {
        Self([x, y,])
    }

    #[inline]
    pub const fn const_into_destr(self) -> CartesianDestr<T> {
        let Self([x, y,]) = self;
        CartesianDestr { x, y, }
    }
}

impl<T> From<CartesianDestr<T>> for Cartesian<T> {
    #[inline]
    fn from(d: CartesianDestr<T>) -> Self {
        Self::from_destr(d)
    }
}

impl<T> From<Cartesian<T>> for CartesianDestr<T> {
    #[inline]
    fn from(d: Cartesian<T>) -> Self {
        d.into_destr()
    }
}
```

#### `builder` Arg

An optional `builder` prefix arg controls whether to generate a builder struct that, at compile-time, ensures that every field is set exactly once before creating the struct.

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
pub struct CartesianBuilder<Z, const S0: bool, const S1: bool>([core::mem::MaybeUninit<Z>; CARTESIAN_LEN]);

pub type NewCartesianBuilder<Z> = CartesianBuilder<Z, false, false>;

impl<T> NewCartesianBuilder<T> {
    // impl notes:
    // need to specify as associated const instead of fn local const, otherwise errors with
    // 'can't use generic parameters from outer item'
    const _UNINIT: core::mem::MaybeUninit<T> = core::mem::MaybeUninit::uninit();

    #[inline]
    pub const fn start() -> Self {
        Self([Self::_UNINIT; CARTESIAN_LEN])
    }
}

// impl notes:
// - cannot use transmute() due to const generic, cannot move out of struct due to Drop.
//   Hopefully rustc is able to optimize away all the 
//   transmute_copy() + core::mem::forget()s and use the same memory.
//   I cannot wait for array transmutes to be stabilized.

impl<Z, const S1: bool> CartesianBuilder<Z, false, S1> {
    #[inline]
    pub const fn with_x(
        mut self,
        val: Z,
    ) -> CartesianBuilder<Z, true, S1> {
        self.0[CARTESIAN_IDX_X] = core::mem::MaybeUninit::new(val);
        unsafe {
            core::mem::transmute_copy::<_, _>(
                &core::mem::ManuallyDrop::new(self)
            )
        }
    }
}

impl<Z, const S0: bool> CartesianBuilder<Z, S0, false> {
    #[inline]
    pub const fn with_y(
        mut self,
        val: Z,
    ) -> CartesianBuilder<Z, S0, true> {
        self.0[CARTESIAN_IDX_Y] = core::mem::MaybeUninit::new(val);
        unsafe {
            core::mem::transmute_copy::<_, _>(
                &core::mem::ManuallyDrop::new(self)
            )
        }
    }
}

impl<Z> CartesianBuilder<Z, true, true> {
    #[inline]
    pub const fn build(self) -> Cartesian<Z> {
        // if not `repr(transparent)`, must use `self.0` instead of `self`,
        // but we always enforce repr(transparent)
        unsafe {
            Cartesian(
                core::mem::transmute_copy::<_, _>(
                    &core::mem::ManuallyDrop::new(self)
                )
            )
        }
    }
}

/// This gets called if the Builder struct was dropped before `self.build()` was called
impl<Z, const S0: bool, const S1: bool> Drop for CartesianBuilder<Z, S0, S1> {
    fn drop(&mut self) {
        if S0 {
            unsafe {
                self.0[CARTESIAN_IDX_X].assume_init_drop();
            }
        }
        if S1 {
            unsafe {
                self.0[CARTESIAN_IDX_Y].assume_init_drop();
            } 
        }
    }
}

impl<Z, const S0: bool, const S1: bool> Clone for CartesianBuilder<Z, S0, S1> where Z: Copy {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0)
    }
}
```

##### Example Builder Usages

###### Attempting to build before setting all fields

```rust,compile_fail,E0599
use generic_array_struct::generic_array_struct;

#[generic_array_struct(builder)]
pub struct Cartesian<T> {
    pub x: T,
    pub y: T,
}

// y has not been set, this fails to compile with
// "method not found in `CartesianBuilder<{integer}, true, false>`"
let pt: Cartesian<u8> = NewCartesianBuilder::start().with_x(1).build();
```

###### Attempting to set a field twice

```rust,compile_fail,E0599
use generic_array_struct::generic_array_struct;

#[generic_array_struct(builder pub)]
pub struct Cartesian<T> {
    pub x: T,
    pub y: T,
}

// attempted to set x twice, this fails to compile with
// "no method named `with_x` found for struct `CartesianBuilder<{integer}, true, true>` in the current scope"
let pt: Cartesian<u8> = NewCartesianBuilder::start().with_x(1).with_y(0).with_x(2).build();
```

###### Proper initialization

```rust
use generic_array_struct::generic_array_struct;

#[generic_array_struct(builder pub(crate))]
pub struct Cartesian<T> {
    pub x: T,
    pub y: T,
}

// proper initialization after setting all fields exactly once
let pt: Cartesian<u8> = NewCartesianBuilder::start().with_x(1).with_y(0).build();
```

#### `trymap` Arg

An optional `trymap` prefix arg controls whether to generate 2 util methods, `try_map_opt` and `try_map_res` for the struct.

```rust
use generic_array_struct::generic_array_struct;

#[generic_array_struct(trymap)]
pub struct Cartesian<Z> {
    pub x: Z,
    pub y: Z,
}
```

expands to

```rust
use generic_array_struct::generic_array_struct;

#[generic_array_struct]
pub struct Cartesian<Z> {
    pub x: Z,
    pub y: Z,
}

// impl notes:
// - cannot use transmute() due to const generic, cannot move out of struct due to Drop.
//   Hopefully rustc is able to optimize away all the 
//   transmute_copy() + core::mem::forget()s and use the same memory.
//   I cannot wait for array transmutes to be stabilized.
// - generate 2 separate methods instead of using `Try` trait so that its compatible
//   with stable rust

impl<T> Cartesian<T> {
    #[inline]
    pub fn try_map_opt<B, F>(
        self,
        mut f: F,
    ) -> Option<Cartesian<B>> where F: FnMut(T) -> Option<B> {
        let mut res: Cartesian<core::mem::MaybeUninit<B>>
            = Cartesian(core::array::from_fn(|_| core::mem::MaybeUninit::uninit()));
        let written = self.0.into_iter().zip(res.0.iter_mut()).try_fold(
            0usize,
            |written, (val, rmut)| {
                rmut.write(f(val).ok_or(written)?);
                Ok(written + 1)
            }
        );
        match written {
            Ok(_) => Some(Cartesian(
                unsafe {
                    core::mem::transmute_copy::<_, _>(
                        &core::mem::ManuallyDrop::new(res.0)
                    )
                }
            )),
            Err(written) => {
                res.0.iter_mut().take(written).for_each(
                    |mu| unsafe { mu.assume_init_drop() }
                );
                None
            }
        }
    }

    #[inline]
    pub fn try_map_res<B, E, F>(
        self,
        mut f: F,
    ) -> Result<Cartesian<B>, E> where F: FnMut(T) -> Result<B, E> {
        let mut res: Cartesian<core::mem::MaybeUninit<B>>
            = Cartesian(core::array::from_fn(|_| core::mem::MaybeUninit::uninit()));
        let written = self.0.into_iter().zip(res.0.iter_mut()).try_fold(
            0usize,
            |written, (val, rmut)| {
                rmut.write(f(val).map_err(|e| (e, written))?);
                Ok(written + 1)
            }
        );
        match written {
            Ok(_) => Ok(Cartesian(
                unsafe {
                    core::mem::transmute_copy::<_, _>(
                        &core::mem::ManuallyDrop::new(res.0)
                    )
                }
            )),
            Err((e, written)) => {
                res.0.iter_mut().take(written).for_each(
                    |mu| unsafe { mu.assume_init_drop() }
                );
                Err(e)
            }
        }
    }
}
```

#### `zip` Arg

An optional `zip` prefix arg controls whether to generate the un/zip util methods.

```rust
use generic_array_struct::generic_array_struct;

#[generic_array_struct(zip)]
pub struct Cartesian<Z> {
    pub x: Z,
    pub y: Z,
}
```

expands to

```rust
use generic_array_struct::generic_array_struct;

#[generic_array_struct]
pub struct Cartesian<Z> {
    pub x: Z,
    pub y: Z,
}

impl<T> Cartesian<T> {
    #[inline]
    pub fn zip<U>(self, Cartesian([u0, u1]): Cartesian<U>) -> Cartesian<(T, U)> {
        let Self([t0, t1]) = self;
        Cartesian([(t0, u0), (t1, u1)])
    }
}

impl<T: Copy> Cartesian<T> {
    #[inline]
    pub const fn const_zip<U: Copy>(self, Cartesian([u0, u1]): Cartesian<U>) -> Cartesian<(T, U)> {
        let Self([t0, t1]) = self;
        Cartesian([(t0, u0), (t1, u1)])
    }
}

impl<T, U> Cartesian<(T, U)> {
    #[inline]
    pub fn unzip(self) -> (Cartesian<T>, Cartesian<U>) {
        let Self([(t0, u0), (t1, u1)]) = self;
        (Cartesian([t0, t1]), Cartesian([u0, u1]))
    }
}

impl<T: Copy, U: Copy> Cartesian<(T, U)> {
    #[inline]
    pub const fn const_unzip(self) -> (Cartesian<T>, Cartesian<U>) {
        let Self([(t0, u0), (t1, u1)]) = self;
        (Cartesian([t0, t1]), Cartesian([u0, u1]))
    }
}
```

#### `all` Arg

Instead of specifying each individual optional prefix arg, a single `all` arg can be specified to enable all of the above.

```rust
use generic_array_struct::generic_array_struct;

#[generic_array_struct(all)]
pub struct Cartesian<Z> {
    pub x: Z,
    pub y: Z,
}
```

is equivalent to

```rust
use generic_array_struct::generic_array_struct;

#[generic_array_struct(builder destr trymap zip)]
pub struct Cartesian<Z> {
    pub x: Z,
    pub y: Z,
}
```

#### `.0` Visibility Attribute Arg

The attribute's final position arg is a [`syn::Visibility`](`syn::Visibility`) that controls the visibility of the resulting `.0` array field. 

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
