use generic_array_struct::generic_array_struct;

#[test]
fn basic() {
    /// doc comment RGB
    #[generic_array_struct]
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
    #[repr(transparent)]
    pub struct Rgb<U> {
        pub r: U,
        pub g: U,
        pub b: U,
    }

    let mut dut: Rgb<u8> = Default::default();

    assert_eq!(RGB_LEN, 3);
    assert_eq!(dut.0, [0, 0, 0]);

    assert_eq!(RGB_IDX_G, 1);
    assert_eq!(*dut.r(), 0);
    assert_eq!(dut.set_r(1), 0);
    assert_eq!(*dut.r(), 1);

    let dut = dut.with_g(2).with_b(3);
    assert_eq!(dut.0, [1, 2, 3]);
}

#[test]
fn const_basic() {
    #[generic_array_struct]
    pub struct Cartesian<D> {
        x: D,
        y: D,
    }

    impl Cartesian<f64> {
        const ORIGIN: Self = Self([0.0; 2]);
    }

    const ONE_COMMA_TWO: Cartesian<f64> = Cartesian::ORIGIN.const_with_x(1.0).const_with_y(2.0);

    assert_eq!(*ONE_COMMA_TWO.x(), 1.0);
    assert_eq!(*ONE_COMMA_TWO.y(), 2.0);
}
