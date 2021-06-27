#![allow(clippy::needless_range_loop)]

use super::default_table_array;
use serde::de;

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

#[cfg(test)]
mod tests {
    use super::*;

    use serde::Deserialize;

    #[derive(Debug, Default, PartialEq, Eq, Deserialize)]
    struct ComplexObj {
        field1: u8,
        field2: String,
        field3: bool,
    }

    #[derive(Debug, PartialEq, Eq, Deserialize)]
    struct TestTableArray<T: Default, const ROW: usize, const COL: usize>(
        #[serde(
            bound(deserialize = "T: Deserialize<'de>"),
            deserialize_with = "deserialize_table_array"
        )]
        [[T; COL]; ROW],
    );

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
