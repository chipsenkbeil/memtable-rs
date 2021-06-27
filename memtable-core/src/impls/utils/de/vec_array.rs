use super::default_array;
use serde::de;

/// Workaround for https://github.com/serde-rs/serde/issues/1937
pub fn deserialize_vec_array<'de, D, T, const N: usize>(
    deserializer: D,
) -> Result<Vec<[T; N]>, D::Error>
where
    D: de::Deserializer<'de>,
    T: de::Deserialize<'de> + Default,
{
    deserializer.deserialize_tuple(N, VecArrayVisitor::<T, N>::default())
}

#[derive(Default)]
struct VecArrayVisitor<T, const N: usize> {
    _marker: core::marker::PhantomData<T>,
}

impl<'de, T, const N: usize> de::Visitor<'de> for VecArrayVisitor<T, N>
where
    T: de::Deserialize<'de> + Default,
{
    type Value = Vec<[T; N]>;

    /// Format a message stating we expect an array of size `N`
    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(formatter, "an array of size {}", N)
    }

    /// Process a sequence into a table array
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut list = Vec::new();
        let mut total_cnt = 0;
        let mut arr_cnt = 0;

        while let Some(next) = seq.next_element::<T>()? {
            // If starting a new array, we need to allocate
            if arr_cnt == 0 {
                list.push(default_array());
            }

            let row = list.len() - 1;
            list[row][arr_cnt] = next;
            total_cnt += 1;
            arr_cnt += 1;

            if arr_cnt == N {
                arr_cnt = 0;
            }
        }

        // If we didn't end on a divisible boundary, we have a problem
        if total_cnt % N > 0 {
            Err(de::Error::invalid_length(total_cnt, &self))

        // Otherwise, we're good to go
        } else {
            Ok(list)
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
    struct TestVecArray<T: Default, const N: usize>(
        #[serde(
            bound(deserialize = "T: Deserialize<'de>"),
            deserialize_with = "deserialize_vec_array"
        )]
        Vec<[T; N]>,
    );

    #[test]
    fn deserialize_vec_array_should_fail_if_not_enough_elements() {
        let s = "[1,2,3,4,5,6,7]";
        let res: serde_json::Result<TestVecArray<usize, 4>> = serde_json::from_str(s);
        assert!(res.is_err());
    }

    #[test]
    fn deserialize_vec_array_should_fail_if_too_many_elements() {
        let s = "[1,2,3,4,5,6,7,8,9]";
        let res: serde_json::Result<TestVecArray<usize, 4>> = serde_json::from_str(s);
        assert!(res.is_err());
    }

    #[test]
    fn deserialize_vec_array_should_correctly_deserialize() {
        let s = "[1,2,3,4,5,6,7,8]";
        let arr: TestVecArray<usize, 4> = serde_json::from_str(s).unwrap();
        assert_eq!(arr, TestVecArray(vec![[1, 2, 3, 4], [5, 6, 7, 8]]));
    }

    #[test]
    fn deserialize_vec_array_should_support_complex_generic_types() {
        let s = concat!(
            "[",
            r#"{"field1":1,"field2":"hello","field3":false}"#,
            ",",
            r#"{"field1":2,"field2":"world","field3":true}"#,
            "]",
        );
        let arr: TestVecArray<ComplexObj, 2> = serde_json::from_str(s).unwrap();
        assert_eq!(
            arr,
            TestVecArray(vec![[
                ComplexObj {
                    field1: 1,
                    field2: "hello".to_string(),
                    field3: false,
                },
                ComplexObj {
                    field1: 2,
                    field2: "world".to_string(),
                    field3: true,
                }
            ],])
        );
    }
}
