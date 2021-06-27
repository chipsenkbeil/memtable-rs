/// Creates a new table in the form of a 2D array initialized element-by-element
/// using the provided function to produce `T`
///
/// Based on [al-jabr's matrix](https://github.com/maplant/al-jabr/blob/master/src/matrix.rs)
/// and workaround due to not having [`Default`] implemented for `[T; N]`
/// (limited to < 32):
///
/// - https://github.com/rust-lang/rust/pull/84838
/// - https://github.com/rust-lang/rust/issues/61956
pub fn make_table_array<T: Sized, const ROW: usize, const COL: usize>(
    f: impl Fn(usize, usize) -> T,
) -> [[T; COL]; ROW] {
    use core::mem::{self, MaybeUninit};
    unsafe {
        let mut data: MaybeUninit<[[T; COL]; ROW]> = MaybeUninit::uninit();
        let data_ptr: *mut [T; COL] = mem::transmute(&mut data);

        for row in 0..ROW {
            let mut tmp: MaybeUninit<[T; COL]> = MaybeUninit::uninit();
            let tmp_ptr: *mut T = mem::transmute(&mut tmp);
            for col in 0..COL {
                tmp_ptr.add(col).write(f(row, col));
            }
            data_ptr.add(row).write(tmp.assume_init());
        }

        data.assume_init()
    }
}

// TODO: This ideally gets cleaned up to just Default::default() for any array
//       given Default implies Sized: https://github.com/rust-lang/rust/pull/84838
//
//       Even before then, there is a different issue about problems with
//       transmuting a generic: https://github.com/rust-lang/rust/issues/61956
pub fn default_table_array<T: Default, const ROW: usize, const COL: usize>() -> [[T; COL]; ROW] {
    make_table_array(|_, _| T::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_table_array_should_correctly_initialize() {
        let arr: [[String; 3]; 2] = make_table_array(|row, col| format!("{},{}", row, col));
        assert_eq!(arr[0][0], "0,0");
        assert_eq!(arr[0][1], "0,1");
        assert_eq!(arr[0][2], "0,2");
        assert_eq!(arr[1][0], "1,0");
        assert_eq!(arr[1][1], "1,1");
        assert_eq!(arr[1][2], "1,2");
    }

    #[test]
    fn default_table_array_should_correctly_initialize() {
        #[derive(Debug, PartialEq, Eq)]
        struct MyField(u8);
        impl Default for MyField {
            fn default() -> Self {
                Self(123)
            }
        }

        let arr: [[MyField; 3]; 2] = default_table_array();
        assert_eq!(arr[0][0], MyField(123));
        assert_eq!(arr[0][1], MyField(123));
        assert_eq!(arr[0][2], MyField(123));
        assert_eq!(arr[1][0], MyField(123));
        assert_eq!(arr[1][1], MyField(123));
        assert_eq!(arr[1][2], MyField(123));
    }
}
