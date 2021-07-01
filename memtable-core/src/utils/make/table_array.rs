use super::array::try_make_array;
use core::{
    convert::Infallible,
    mem::{self, MaybeUninit},
};

/// Creates a new array initialized element-by-element using the provided
/// function to produce `T`
///
/// Based on:
///
/// - [al-jabr's matrix](https://github.com/maplant/al-jabr/blob/master/src/matrix.rs)
/// - [serde_array's deserialize](https://github.com/Kromey/serde_arrays/blob/1d14bfd2c7b7aaf418776f1fe88a0e2537bfcae4/src/lib.rs#L156)
///
/// Acts as a workaround due to not having [`Default`] implemented for `[T; N]`
/// (limited to < 32):
///
/// - https://github.com/rust-lang/rust/pull/84838
/// - https://github.com/rust-lang/rust/issues/61956
pub fn try_make_table_array<T: Sized, E, const ROW: usize, const COL: usize>(
    mut f: impl FnMut(usize, usize) -> Result<T, E>,
) -> Result<[[T; COL]; ROW], E> {
    unsafe {
        let mut data: MaybeUninit<[[T; COL]; ROW]> = MaybeUninit::uninit();
        let data_ptr: *mut [T; COL] = mem::transmute(&mut data);
        let mut cnt = 0;
        let mut err = None;

        for row in 0..ROW {
            match try_make_array(|col| f(row, col)) {
                Ok(x) => {
                    data_ptr.add(row).write(x);
                    cnt += 1;
                }
                Err(x) => {
                    err = Some((cnt, x));
                    break;
                }
            }
        }

        if let Some((cnt, x)) = err {
            for i in (0..cnt).rev() {
                data_ptr.add(i).drop_in_place();
            }
            return Err(x);
        }

        Ok(data.assume_init())
    }
}

/// Like [`try_make_table_array`], but uses an element allocator that is guaranteed
/// to succeed; therefore, this 2D array allocator will also be guaranteed to succeed
pub fn make_table_array<T: Sized, const ROW: usize, const COL: usize>(
    mut f: impl FnMut(usize, usize) -> T,
) -> [[T; COL]; ROW] {
    let res: Result<[[T; COL]; ROW], Infallible> = try_make_table_array(|row, col| Ok(f(row, col)));
    res.expect("BUG: This should never fail! If you're seeing this, there may be a memory leak!")
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
    use std::format;
    use std::{boxed::Box, string::String};

    #[derive(Debug, PartialEq, Eq)]
    struct ComplexObj {
        row: usize,
        col: usize,
        // Heap allocation via Vec<u8> underneath
        text: String,
        // Heap allocation via Box<...>
        box_str: Box<&'static str>,
    }

    impl ComplexObj {
        pub fn new(
            row: usize,
            col: usize,
            text: impl Into<String>,
            box_str: impl Into<Box<&'static str>>,
        ) -> Self {
            Self {
                row,
                col,
                text: text.into(),
                box_str: box_str.into(),
            }
        }
    }

    #[test]
    fn try_make_table_array_should_correctly_initialize_if_all_element_calls_succeed() {
        let arr: Result<[[String; 3]; 2], Infallible> =
            try_make_table_array(|row, col| Ok(format!("{},{}", row, col)));
        let arr = arr.unwrap();
        assert_eq!(arr[0][0], "0,0");
        assert_eq!(arr[0][1], "0,1");
        assert_eq!(arr[0][2], "0,2");
        assert_eq!(arr[1][0], "1,0");
        assert_eq!(arr[1][1], "1,1");
        assert_eq!(arr[1][2], "1,2");
    }

    #[test]
    fn try_make_table_array_should_correctly_deallocate_if_an_element_call_fails() {
        let arr: Result<[[String; 3]; 2], &'static str> = try_make_table_array(|row, col| {
            if row == 1 && col == 1 {
                Err("Failure!")
            } else {
                Ok(format!("{},{}", row, col))
            }
        });
        assert_eq!(arr.unwrap_err(), "Failure!");
    }

    #[test]
    fn try_make_table_array_should_support_complex_objects_with_heap_allocations() {
        let arr: Result<[[ComplexObj; 3]; 2], Infallible> = try_make_table_array(|row, col| {
            Ok(ComplexObj::new(
                row,
                col,
                format!("{},{}", row, col),
                "complex",
            ))
        });
        let arr = arr.unwrap();
        assert_eq!(arr[0][0], ComplexObj::new(0, 0, "0,0", "complex"));
        assert_eq!(arr[0][1], ComplexObj::new(0, 1, "0,1", "complex"));
        assert_eq!(arr[0][2], ComplexObj::new(0, 2, "0,2", "complex"));
        assert_eq!(arr[1][0], ComplexObj::new(1, 0, "1,0", "complex"));
        assert_eq!(arr[1][1], ComplexObj::new(1, 1, "1,1", "complex"));
        assert_eq!(arr[1][2], ComplexObj::new(1, 2, "1,2", "complex"));
    }

    #[test]
    fn try_make_table_array_should_support_deallocating_complex_objects_on_failure() {
        let arr: Result<[[ComplexObj; 3]; 2], &'static str> = try_make_table_array(|row, col| {
            if row == 1 && col == 1 {
                Err("Failure!")
            } else {
                Ok(ComplexObj::new(
                    row,
                    col,
                    format!("{},{}", row, col),
                    "complex",
                ))
            }
        });
        assert_eq!(arr.unwrap_err(), "Failure!");
    }

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
