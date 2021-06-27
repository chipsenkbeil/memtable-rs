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
pub fn try_make_array<T: Sized, E, const N: usize>(
    mut f: impl FnMut(usize) -> Result<T, E>,
) -> Result<[T; N], E> {
    unsafe {
        let mut data: MaybeUninit<[T; N]> = MaybeUninit::uninit();
        let data_ptr: *mut T = mem::transmute(&mut data);
        let mut cnt = 0;
        let mut err = None;

        // Loop through our ptr to the future array and allocate a single
        // element at a time, assigning it to the next contiguous block
        // within the array
        for i in 0..N {
            match f(i) {
                Ok(x) => {
                    data_ptr.add(i).write(x);
                    cnt += 1;
                }
                Err(x) => {
                    err = Some((cnt, x));
                    break;
                }
            }
        }

        // Didn't finish the array, so we need to remove everything we allocated
        if let Some((cnt, x)) = err {
            for i in (0..cnt).rev() {
                data_ptr.add(i).drop_in_place();
            }
            return Err(x);
        }

        Ok(data.assume_init())
    }
}

/// Like [`try_make_array`], but uses an element allocator that is guaranteed
/// to succeed; therefore, this array allocator will also be guaranteed to succeed
pub fn make_array<T: Sized, const N: usize>(mut f: impl FnMut(usize) -> T) -> [T; N] {
    let res: Result<[T; N], Infallible> = try_make_array(|i| Ok(f(i)));
    res.expect("BUG: This should never fail! If you're seeing this, there may be a memory leak!")
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

    #[derive(Debug, PartialEq, Eq)]
    struct ComplexObj {
        idx: usize,
        // Heap allocation via Vec<u8> underneath
        text: String,
        // Heap allocation via Box<...>
        box_str: Box<&'static str>,
    }

    impl ComplexObj {
        pub fn new(
            idx: usize,
            text: impl Into<String>,
            box_str: impl Into<Box<&'static str>>,
        ) -> Self {
            Self {
                idx,
                text: text.into(),
                box_str: box_str.into(),
            }
        }
    }

    #[test]
    fn try_make_array_should_correctly_initialize_if_all_element_calls_succeed() {
        let arr: Result<[String; 2], Infallible> = try_make_array(|i| Ok(format!("{}", i)));
        let arr = arr.unwrap();
        assert_eq!(arr[0], "0");
        assert_eq!(arr[1], "1");
    }

    #[test]
    fn try_make_array_should_correctly_deallocate_if_an_element_call_fails() {
        let arr: Result<[String; 2], &'static str> = try_make_array(|i| {
            if i == 0 {
                Ok(format!("{}", i))
            } else {
                Err("Failure!")
            }
        });
        assert_eq!(arr.unwrap_err(), "Failure!");
    }

    #[test]
    fn try_make_array_should_support_complex_objects_with_heap_allocations() {
        let arr: Result<[ComplexObj; 3], Infallible> =
            try_make_array(|idx| Ok(ComplexObj::new(idx, format!("{}", idx), "complex")));
        let arr = arr.unwrap();
        assert_eq!(arr[0], ComplexObj::new(0, "0", "complex"));
        assert_eq!(arr[1], ComplexObj::new(1, "1", "complex"));
        assert_eq!(arr[2], ComplexObj::new(2, "2", "complex"));
    }

    #[test]
    fn try_make_array_should_support_deallocating_complex_objects_on_failure() {
        let arr: Result<[ComplexObj; 3], &'static str> = try_make_array(|idx| {
            if idx == 1 {
                Err("Failure!")
            } else {
                Ok(ComplexObj::new(idx, format!("{}", idx), "complex"))
            }
        });
        assert_eq!(arr.unwrap_err(), "Failure!");
    }

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
