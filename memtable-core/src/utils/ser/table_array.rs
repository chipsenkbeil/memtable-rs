use serde::ser;

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

    #[allow(clippy::needless_range_loop)]
    for row in 0..ROW {
        for col in 0..COL {
            tup.serialize_element(&value[row][col])?;
        }
    }
    tup.end()
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde::Serialize;

    #[derive(Debug, Default, PartialEq, Eq, Serialize)]
    struct ComplexObj {
        field1: u8,
        field2: String,
        field3: bool,
    }

    #[derive(Debug, PartialEq, Eq, Serialize)]
    struct TestTableArray<T: Default, const ROW: usize, const COL: usize>(
        #[serde(
            bound(serialize = "T: Serialize"),
            serialize_with = "serialize_table_array"
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
}
