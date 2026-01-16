#![cfg_attr(not(test), no_std)]

use generic_array_struct::generic_array_struct;

/// A RGB color triple
#[generic_array_struct(all pub)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Rgb<T> {
    /// red
    r: T,

    /// green
    g: T,

    /// blue
    b: T,
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
    fn consts_sc() {
        [
            [RGB_LEN, RgbU8::LEN],
            [RGB_IDX_B, RgbU8::IDX_B],
            [RGB_IDX_G, RgbU8::IDX_G],
            [RGB_IDX_R, RgbU8::IDX_R],
        ]
        .into_iter()
        .for_each(|[a, b]| assert_eq!(a, b));
    }

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
        // a gets dropped here. If Builder not mem::forgotten,
        // double free will occur and segfault
    }

    #[test]
    fn mem_safety_drop_builder() {
        let r = Rc::new(1);

        let a = NewRgbBuilder::start().with_r(r.clone()).with_g(r);
        eprintln!("{:#?}", a.0);
        // partially initialized a gets dropped here.
        // Make sure Builder `Drop` impl doesnt
        // attempt to drop uninitialized memory.
        // If so, this will segfault for attempting
        // to decrement nonexistent Rc.
    }

    #[test]
    fn clone_builder() {
        let full_r = NewRgbBuilder::start().with_r(255u8);
        let yellow = full_r.clone().with_g(255).with_b(0).build();
        let purple = full_r.with_g(0).with_b(255).build();
        eprintln!("{yellow:#?} {purple:#?}");
    }

    #[test]
    fn destr_debug() {
        eprintln!("{:#?}", BLACK.const_into_destr());
    }

    #[test]
    fn destr_rt() {
        [Rgb::blue(), Rgb::green(), Rgb::red(), Rgb::white(), BLACK]
            .into_iter()
            .for_each(|c| {
                let destr = c.into_destr();
                assert_eq!(destr, c.into());

                let roundtripped = Rgb::from_destr(destr);
                assert_eq!(roundtripped, destr.into());

                assert_eq!(c, roundtripped);
            });
    }

    #[test]
    fn mem_safety_try_map_fail() {
        const FAIL: u8 = 67;
        const SRC: Rgb<u8> = Rgb([0, FAIL, 0]);

        let none_on_g = |x: u8| (x != FAIL).then_some(vec![x]);
        let err_on_g = |x: u8| none_on_g(x).ok_or(x);

        let opt = SRC.try_map_opt(none_on_g);
        assert_eq!(opt, None);

        let res = SRC.try_map_res(err_on_g);
        assert_eq!(res, Err(FAIL));

        // if the initialized [0] MaybeUninit in try_map_*
        // isn't cleaned up properly then miri will detect a mem leak for the vecs
    }

    #[test]
    fn mem_safety_try_map_fail_2() {
        const FAIL: u8 = 67;

        let src = Rgb([vec![0], vec![FAIL], vec![0]]);

        let none_on_g = |x: Vec<u8>| (x[0] != FAIL).then_some(x);
        let err_on_g = |x: Vec<u8>| none_on_g(x).ok_or(());

        let opt = src.clone().try_map_opt(none_on_g);
        assert_eq!(opt, None);

        let res = src.try_map_res(err_on_g);
        assert_eq!(res, Err(()));

        // if the vecs in src aren't cleaned up properly
        // then there will be a double free when dropping both src
        // and the MaybeUninits in try_map_*
    }

    #[test]
    fn mem_safety_try_map_success() {
        let r = Rc::new(1);
        let src = Rgb(core::array::from_fn(|_| r.clone()));

        let id_clone_opt = |x: Rc<u8>| Some(x.clone());
        let id_clone_res = |x: Rc<u8>| Ok::<_, ()>(x.clone());

        assert_eq!(src.clone().try_map_opt(id_clone_opt).unwrap(), src);
        assert_eq!(src.clone().try_map_res(id_clone_res).unwrap(), src);

        // idk, just using Rc here to check for any weirdness
    }

    #[test]
    fn try_map_basic() {
        const SRC: Rgb<u8> = Rgb([1, 2, 3]);

        let f_opt_id = |x: u8| Some(x);
        let f_res_id = |x: u8| Ok::<_, ()>(x);

        assert_eq!(SRC.try_map_opt(f_opt_id).unwrap(), SRC);
        assert_eq!(SRC.try_map_res(f_res_id).unwrap(), SRC);
    }

    #[test]
    fn zip_unzip_basic() {
        const T: Rgb<u8> = Rgb([0, 1, 2]);
        const U: Rgb<f64> = Rgb([0.0, 1.0, 2.0]);
        const TU: Rgb<(u8, f64)> = T.const_zip(U);
        const TU_UNZ: (Rgb<u8>, Rgb<f64>) = TU.const_unzip();

        let v: Rgb<Vec<u8>> = Rgb([vec![0], vec![1], vec![2]]);

        assert_eq!(TU, Rgb(core::array::from_fn(|i| (i as u8, i as f64))));
        assert_eq!(TU, T.zip(U));
        assert_eq!(
            T.zip(v),
            Rgb(core::array::from_fn(|i| (i as u8, vec![i as u8])))
        );

        assert_eq!(TU_UNZ, (T, U));
        assert_eq!(TU_UNZ, TU.unzip());
    }
}
