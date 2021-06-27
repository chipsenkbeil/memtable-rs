use super::default_array;
use serde::de;

/// Workaround for https://github.com/serde-rs/serde/issues/1937
pub fn deserialize_array<'de, D, T, const N: usize>(deserializer: D) -> Result<[T; N], D::Error>
where
    D: de::Deserializer<'de>,
    T: de::Deserialize<'de> + Default,
{
    deserializer.deserialize_tuple(N, ArrayVisitor::<T, N>::default())
}

#[derive(Default)]
struct ArrayVisitor<T, const N: usize> {
    _marker: core::marker::PhantomData<T>,
}

impl<'de, T, const N: usize> de::Visitor<'de> for ArrayVisitor<T, N>
where
    T: de::Deserialize<'de> + Default,
{
    type Value = [T; N];

    /// Format a message stating we expect an array of size `N`
    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(formatter, "an array of size {}", N)
    }

    /// Process a sequence into a table array
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut arr = default_array::<T, N>();
        let mut cnt = 0;

        for i in 0..N {
            let next = seq.next_element::<T>()?;

            // Assign the element to our array if we have it
            if let Some(x) = next {
                arr[i] = x;
                cnt += 1;

            // Otherwise, we have a bad sequence
            } else {
                return Err(de::Error::invalid_length(cnt, &self));
            }
        }

        // If we still have more elements, there's a problem
        if seq.next_element::<T>()?.is_some() {
            Err(de::Error::invalid_length(cnt + 1, &self))

        // Otherwise, we're good to go
        } else {
            Ok(arr)
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
    struct TestArray<T: Default, const N: usize>(
        #[serde(
            bound(deserialize = "T: Deserialize<'de>"),
            deserialize_with = "deserialize_array"
        )]
        [T; N],
    );

    #[test]
    fn deserialize_array_should_fail_if_not_enough_elements() {
        let s = "[1,2,3,4,5,6,7]";
        let res: serde_json::Result<TestArray<usize, 8>> = serde_json::from_str(s);
        assert!(res.is_err());
    }

    #[test]
    fn deserialize_array_should_fail_if_too_many_elements() {
        let s = "[1,2,3,4,5,6,7,8,9]";
        let res: serde_json::Result<TestArray<usize, 8>> = serde_json::from_str(s);
        assert!(res.is_err());
    }

    #[test]
    fn deserialize_array_should_correctly_deserialize() {
        let s = "[1,2,3,4,5,6,7,8]";
        let arr: TestArray<usize, 8> = serde_json::from_str(s).unwrap();
        assert_eq!(arr, TestArray([1, 2, 3, 4, 5, 6, 7, 8]));
    }

    #[test]
    fn deserialize_array_should_support_complex_generic_types() {
        let s = concat!(
            "[",
            r#"{"field1":1,"field2":"hello","field3":false}"#,
            ",",
            r#"{"field1":2,"field2":"world","field3":true}"#,
            "]",
        );
        let arr: TestArray<ComplexObj, 2> = serde_json::from_str(s).unwrap();
        assert_eq!(
            arr,
            TestArray([
                ComplexObj {
                    field1: 1,
                    field2: "hello".to_string(),
                    field3: false,
                },
                ComplexObj {
                    field1: 2,
                    field2: "world".to_string(),
                    field3: true,
                },
            ])
        );
    }
}
