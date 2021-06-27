#[cfg(feature = "serde-1")]
pub use serde_table_array::{deserialize_table_array, serialize_table_array};

#[cfg(feature = "serde-1")]
#[allow(clippy::needless_range_loop)]
mod serde_table_array {
    use super::default_table_array;

    use serde::{de, ser};

    /// Workaround for https://github.com/serde-rs/serde/issues/1937
    pub fn serialize_table_array<S, T, const ROW: usize, const COL: usize>(
        value: &[[T; COL]; ROW],
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
        T: ser::Serialize,
    {
        use ser::SerializeTuple;
        let mut tup = serializer.serialize_tuple(ROW * COL)?;
        for row in 0..ROW {
            for col in 0..COL {
                tup.serialize_element(&value[row][col])?;
            }
        }
        tup.end()
    }

    /// Workaround for https://github.com/serde-rs/serde/issues/1937
    pub fn deserialize_table_array<'de, D, T, const ROW: usize, const COL: usize>(
        deserializer: D,
    ) -> Result<[[T; COL]; ROW], D::Error>
    where
        D: de::Deserializer<'de>,
        T: de::Deserialize<'de> + Default,
    {
        deserializer.deserialize_tuple(ROW * COL, TableArrayVisitor::<T, ROW, COL>::default())
    }

    #[derive(Default)]
    struct TableArrayVisitor<T, const ROW: usize, const COL: usize> {
        _marker: core::marker::PhantomData<T>,
    }

    impl<'de, T, const ROW: usize, const COL: usize> de::Visitor<'de> for TableArrayVisitor<T, ROW, COL>
    where
        T: de::Deserialize<'de> + Default,
    {
        type Value = [[T; COL]; ROW];

        /// Format a message stating we expect an array of size `N`
        fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
            write!(formatter, "an array of size {}", ROW * COL)
        }

        /// Process a sequence into a table array
        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut table = default_table_array::<T, ROW, COL>();
            let mut cnt = 0;

            for row in 0..ROW {
                for col in 0..COL {
                    let next = seq.next_element::<T>()?;

                    // Assign the element to our array if we have it
                    if let Some(x) = next {
                        table[row][col] = x;
                        cnt += 1;

                    // Otherwise, we have a bad sequence
                    } else {
                        return Err(de::Error::invalid_length(cnt, &self));
                    }
                }
            }

            // If we still have more elements, there's a problem
            if seq.next_element::<T>()?.is_some() {
                Err(de::Error::invalid_length(cnt + 1, &self))

            // Otherwise, we're good to go
            } else {
                Ok(table)
            }
        }
    }
}

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

    #[cfg(feature = "serde-1")]
    mod serde_table_array {
        use super::*;
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
        struct ComplexObj {
            field1: u8,
            field2: String,
            field3: bool,
        }

        #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
        struct TestTableArray<T: Default, const ROW: usize, const COL: usize>(
            #[serde(
                bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"),
                serialize_with = "serialize_table_array",
                deserialize_with = "deserialize_table_array"
            )]
            [[T; COL]; ROW],
        );

        #[test]
        fn serialize_table_array_should_correctly_serialize() {
            let arr = TestTableArray([[1, 2, 3, 4], [5, 6, 7, 8]]);
            let s = serde_json::to_string(&arr).unwrap();
            assert_eq!(s, "[1,2,3,4,5,6,7,8]");
        }

        #[test]
        fn serialize_table_array_should_support_complex_generic_types() {
            let arr = TestTableArray([
                [ComplexObj {
                    field1: 1,
                    field2: "hello".to_string(),
                    field3: false,
                }],
                [ComplexObj {
                    field1: 2,
                    field2: "world".to_string(),
                    field3: true,
                }],
            ]);
            let s = serde_json::to_string(&arr).unwrap();
            assert_eq!(
                s,
                concat!(
                    "[",
                    r#"{"field1":1,"field2":"hello","field3":false}"#,
                    ",",
                    r#"{"field1":2,"field2":"world","field3":true}"#,
                    "]",
                )
            );
        }

        #[test]
        fn deserialize_table_array_should_fail_if_not_enough_elements() {
            let s = "[1,2,3,4,5,6,7]";
            let res: serde_json::Result<TestTableArray<usize, 2, 4>> = serde_json::from_str(s);
            assert!(res.is_err());
        }

        #[test]
        fn deserialize_table_array_should_fail_if_too_many_elements() {
            let s = "[1,2,3,4,5,6,7,8,9]";
            let res: serde_json::Result<TestTableArray<usize, 2, 4>> = serde_json::from_str(s);
            assert!(res.is_err());
        }

        #[test]
        fn deserialize_table_array_should_correctly_deserialize() {
            let s = "[1,2,3,4,5,6,7,8]";
            let arr: TestTableArray<usize, 2, 4> = serde_json::from_str(s).unwrap();
            assert_eq!(arr, TestTableArray([[1, 2, 3, 4], [5, 6, 7, 8]]));
        }

        #[test]
        fn deserialize_table_array_should_support_complex_generic_types() {
            let s = concat!(
                "[",
                r#"{"field1":1,"field2":"hello","field3":false}"#,
                ",",
                r#"{"field1":2,"field2":"world","field3":true}"#,
                "]",
            );
            let arr: TestTableArray<ComplexObj, 2, 1> = serde_json::from_str(s).unwrap();
            assert_eq!(
                arr,
                TestTableArray([
                    [ComplexObj {
                        field1: 1,
                        field2: "hello".to_string(),
                        field3: false,
                    }],
                    [ComplexObj {
                        field1: 2,
                        field2: "world".to_string(),
                        field3: true,
                    }],
                ])
            );
        }
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
