/// Creates a new array initialized element-by-element using the provided
/// function to produce `T`
///
/// Based on [al-jabr's matrix](https://github.com/maplant/al-jabr/blob/master/src/matrix.rs)
/// and workaround due to not having [`Default`] implemented for `[T; N]`
/// (limited to < 32):
///
/// - https://github.com/rust-lang/rust/pull/84838
/// - https://github.com/rust-lang/rust/issues/61956
pub fn make_array<T: Sized, const N: usize>(f: impl Fn(usize) -> T) -> [T; N] {
    use core::mem::{self, MaybeUninit};
    unsafe {
        let mut data: MaybeUninit<[T; N]> = MaybeUninit::uninit();
        let data_ptr: *mut T = mem::transmute(&mut data);

        for i in 0..N {
            data_ptr.add(i).write(f(i));
        }

        data.assume_init()
    }
}

// TODO: This ideally gets cleaned up to just Default::default() for any array
//       given Default implies Sized: https://github.com/rust-lang/rust/pull/84838
//
//       Even before then, there is a different issue about problems with
//       transmuting a generic: https://github.com/rust-lang/rust/issues/61956
pub fn default_array<T: Default, const N: usize>() -> [T; N] {
    make_array(|_| T::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_array_should_correctly_initialize() {
        let arr: [String; 2] = make_array(|i| format!("{}", i));
        assert_eq!(arr[0], "0");
        assert_eq!(arr[1], "1");
    }

    #[test]
    fn default_array_should_correctly_initialize() {
        #[derive(Debug, PartialEq, Eq)]
        struct MyField(u8);
        impl Default for MyField {
            fn default() -> Self {
                Self(123)
            }
        }

        let arr: [MyField; 2] = default_array();
        assert_eq!(arr[0], MyField(123));
        assert_eq!(arr[1], MyField(123));
    }
}
