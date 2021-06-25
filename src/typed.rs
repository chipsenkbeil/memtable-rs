use crate::*;

use paste::paste;
use std::{
    fmt,
    ops::{Deref, DerefMut},
};

macro_rules! impl_table {
    ($name:ident $cell:ident $($variant:ident)+) => {
        paste! {
            #[derive(Clone, Debug, PartialEq, Eq, Hash)]
            #[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
            pub enum [<$name Error>] {
                $([<Missing $variant Cell>] { row: usize }),+
            }

            impl fmt::Display for [<$name Error>] {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    match self {
                        $(
                            Self::[<Missing $variant Cell>] { row } =>
                                write!(
                                    f,
                                    concat!("Missing ", stringify!($variant), " at row {}"),
                                    row,
                                )?,
                        )+
                    }

                    Ok(())
                }
            }

            impl std::error::Error for [<$name Error>] {}

            #[derive(Clone, Debug, Eq, PartialEq)]
            #[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
            pub struct $name<$($variant),+>(Table<$cell<$($variant),+>>);

            impl<$($variant),+> AsRef<Table<$cell<$($variant),+>>> for $name<$($variant),+> {
                fn as_ref(&self) -> &Table<$cell<$($variant),+>> {
                    &self.0
                }
            }

            impl<$($variant),+> AsMut<Table<$cell<$($variant),+>>> for $name<$($variant),+> {
                fn as_mut(&mut self) -> &mut Table<$cell<$($variant),+>> {
                    &mut self.0
                }
            }

            impl<$($variant),+> From<$name<$($variant),+>> for Table<$cell<$($variant),+>> {
                fn from(x: $name<$($variant),+>) -> Self {
                    x.0
                }
            }

            impl<$($variant),+> Default for $name<$($variant),+> {
                fn default() -> Self {
                    Self(Default::default())
                }
            }

            impl<$($variant),+> $name<$($variant),+> {
                /// Creates an empty instance of this table
                pub fn new() -> Self {
                    Self::default()
                }

                /// Converts a label like "A" or "B" to the column index,
                /// or returns none if label does not match a column
                #[allow(unused_assignments)]
                pub fn column_label_to_index(label: &str) -> Option<usize> {
                    let mut index = 0;

                    $(
                        if label == stringify!($variant) {
                            return Some(index);
                        } else {
                            index += 1;
                        }
                    )+

                    None
                }

                /// Naive case-insensitive version that converts label to uppercase
                /// first before being tried
                pub fn case_insensitive_column_label_to_index(label: &str) -> Option<usize> {
                    Self::column_label_to_index(label.to_uppercase().as_str())
                }

                /// Returns an iterator of tuples containing refs to each row's data
                ///
                /// Will return an error if any cell in a row is missing or the wrong type
                pub fn rows(&self) -> impl Iterator<Item = Result<($(&$variant),+), [<$name Error>]>> {
                    (0..self.0.row_cnt()).into_iter().map(move |idx| self.row(idx))
                }

                /// Returns an iterator of tuples containing refs to each row's data wrapped in `Option`.
                ///
                /// If a cell is missing or is the wrong type, `Option::None`
                /// will be returned instead
                pub fn rows_opt(&self) -> impl Iterator<Item = ($(Option<&$variant>),+)> {
                    (0..self.0.row_cnt()).into_iter().map(move |idx| self.row_opt(idx))
                }

                /// Returns a tuple containing refs to row's data
                ///
                /// Will return an error if any cell is missing or the wrong type
                pub fn row(&self, idx: usize) -> Result<($(&$variant),+), [<$name Error>]> {
                    let ($([<$variant:lower>]),+) = self.row_opt(idx);
                    Ok(($(
                        [<$variant:lower>].ok_or_else(||
                            [<$name Error>]::[<Missing $variant Cell>] { row: idx }
                        )?
                    ),+))
                }

                /// Returns a tuple containing refs to the row's data wrapped in `Option`.
                ///
                /// If a cell is missing or is the wrong type, `Option::None`
                /// will be returned instead
                pub fn row_opt(&self, idx: usize) -> ($(Option<&$variant>),+) {
                    ($(self.[<get_ $variant:snake _cell>](idx)),+)
                }

                /// Returns a tuple containing the row's data
                ///
                /// Will return an error if any cell in the row is missing or the wrong type
                pub fn into_row(self, idx: usize) -> Result<($($variant),+), [<$name Error>]> {
                    let ($([<$variant:lower>]),+) = self.into_row_opt(idx);
                    Ok(($(
                        [<$variant:lower>].ok_or_else(||
                            [<$name Error>]::[<Missing $variant Cell>] { row: idx }
                        )?
                    ),+))
                }

                /// Returns a tuple containing the row's data wrapped in `Option`.
                ///
                /// If a cell is missing or is the wrong type, `Option::None`
                /// will be returned instead
                pub fn into_row_opt(mut self, idx: usize) -> ($(Option<$variant>),+) {
                    ($(self.[<remove_ $variant:snake _cell>](idx)),+)
                }

                /// Inserts a new row into the table at the given position, shifting down
                /// all rows after it
                pub fn insert_row(&mut self, idx: usize, cells: ($($variant),+)) {
                    let ($([<$variant:lower>]),+) = cells;
                    self.0.insert_row(idx, vec![
                        $($cell::$variant([<$variant:lower>])),+
                    ]);
                }

                /// Pushes a row to the end of the table
                pub fn push_row(&mut self, cells: ($($variant),+)) {
                    self.insert_row(self.row_cnt(), cells)
                }

                /// Removes the row at the specified position, shifting up all rows after it
                ///
                /// If the row does not exist, then an none will be returned
                pub fn remove_row(&mut self, idx: usize) -> Option<Result<($($variant),+), [<$name Error>]>> {
                    self.remove_row_opt(idx).map(|($([<$variant:lower>]),+)| {
                        Ok(($(
                            [<$variant:lower>].ok_or_else(||
                                [<$name Error>]::[<Missing $variant Cell>] { row: idx }
                            )?
                        ),+))
                    })
                }

                /// Removes the row at the specified position, shifting up all rows after it
                ///
                /// If the row does not exist, then an none will be returned
                ///
                /// If a cell is missing or is the wrong type, `Option::None`
                /// will be returned instead
                pub fn remove_row_opt(&mut self, idx: usize) -> Option<($(Option<$variant>),+)> {
                    if idx < self.row_cnt() {
                        Some(($(self.[<remove_ $variant:snake _cell>](idx)),+))
                    } else {
                        None
                    }
                }

                $(
                    #[doc = "Returns reference to the cell in column " $variant " found at the specified row"]
                    pub fn [<get_ $variant:snake _cell>](&self, row: usize) -> Option<&$variant> {
                        let col = Self::column_label_to_index(stringify!($variant))
                            .expect("Missing variant in label mapping");
                        self.0.get_cell(row, col).and_then($cell::[<as_ $variant:snake>])
                    }

                    #[doc = "Returns mutable reference to the cell in column " $variant " found at the specified row"]
                    pub fn [<get_mut_ $variant:snake _cell>](&mut self, row: usize) -> Option<&mut $variant> {
                        let col = Self::column_label_to_index(stringify!($variant))
                            .expect("Missing variant in label mapping");
                        self.0.get_mut_cell(row, col).and_then($cell::[<as_mut_ $variant:snake>])
                    }

                    #[doc = "Removes cell in column " $variant " found at the specified row"]
                    pub fn [<remove_ $variant:snake _cell>](&mut self, row: usize) -> Option<$variant> {
                        let col = Self::column_label_to_index(stringify!($variant))
                            .expect("Missing variant in label mapping");
                        self.0.remove_cell(row, col).and_then($cell::[<into_ $variant:snake>])
                    }

                    #[doc = "Returns an iterator of refs to `" $variant "` through column " $variant " in the table"]
                    pub fn [<column_ $variant:snake>](&self, idx: usize) -> impl Iterator<Item = &$variant> {
                        self.0.column(idx).filter_map($cell::[<as_ $variant:snake>])
                    }

                    #[doc = "Returns an iterator of `" $variant "` column " $variant " in the table"]
                    pub fn [<into_column_ $variant:snake>](self, idx: usize) -> impl Iterator<Item = $variant> {
                        self.0.into_column(idx).filter_map($cell::[<into_ $variant:snake>])
                    }
                )+
            }

            impl<$($variant),+> Deref for $name<$($variant),+> {
                type Target = Table<$cell<$($variant),+>>;

                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }

            impl<$($variant),+> DerefMut for $name<$($variant),+> {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.0
                }
            }
        }
    };
}

impl_table!(Table2 Cell2 A B);
impl_table!(Table3 Cell3 A B C);
impl_table!(Table4 Cell4 A B C D);
impl_table!(Table5 Cell5 A B C D E);
impl_table!(Table6 Cell6 A B C D E F);
impl_table!(Table7 Cell7 A B C D E F G);
impl_table!(Table8 Cell8 A B C D E F G H);
impl_table!(Table9 Cell9 A B C D E F G H I);
impl_table!(Table10 Cell10 A B C D E F G H I J);
impl_table!(Table11 Cell11 A B C D E F G H I J K);
impl_table!(Table12 Cell12 A B C D E F G H I J K L);
impl_table!(Table13 Cell13 A B C D E F G H I J K L M);
impl_table!(Table14 Cell14 A B C D E F G H I J K L M N);
impl_table!(Table15 Cell15 A B C D E F G H I J K L M N O);
impl_table!(Table16 Cell16 A B C D E F G H I J K L M N O P);
impl_table!(Table17 Cell17 A B C D E F G H I J K L M N O P Q);
impl_table!(Table18 Cell18 A B C D E F G H I J K L M N O P Q R);
impl_table!(Table19 Cell19 A B C D E F G H I J K L M N O P Q R S);
impl_table!(Table20 Cell20 A B C D E F G H I J K L M N O P Q R S T);
impl_table!(Table21 Cell21 A B C D E F G H I J K L M N O P Q R S T U);
impl_table!(Table22 Cell22 A B C D E F G H I J K L M N O P Q R S T U V);
impl_table!(Table23 Cell23 A B C D E F G H I J K L M N O P Q R S T U V W);
impl_table!(Table24 Cell24 A B C D E F G H I J K L M N O P Q R S T U V W X);
impl_table!(Table25 Cell25 A B C D E F G H I J K L M N O P Q R S T U V W X Y);
impl_table!(Table26 Cell26 A B C D E F G H I J K L M N O P Q R S T U V W X Y Z);

#[cfg(test)]
mod tests {
    use super::*;

    /// Test version of Table2
    type T2 = Table2<u8, String>;

    #[test]
    fn column_label_to_index_should_return_appropriate_index() {
        assert_eq!(T2::column_label_to_index("A"), Some(0));
        assert_eq!(T2::column_label_to_index("B"), Some(1));
        assert!(T2::column_label_to_index("C").is_none());
    }

    #[test]
    fn rows_should_return_typed_version_of_each_row() {
        let mut table = T2::new();
        table.push_row((0, "a".to_string()));
        table.push_row((1, "b".to_string()));

        let mut rows = table.rows();
        assert_eq!(rows.next(), Some(Ok((&0, &"a".to_string()))));
        assert_eq!(rows.next(), Some(Ok((&1, &"b".to_string()))));
        assert!(rows.next().is_none());
    }

    #[test]
    fn rows_should_return_error_if_a_row_is_missing_data() {
        let mut table = T2::new();
        table.push_row((0, "a".to_string()));
        table.push_row((1, "b".to_string()));

        // Remove "a"
        table.0.remove_cell(0, 1);

        let mut rows = table.rows();
        assert_eq!(rows.next(), Some(Err(Table2Error::MissingBCell { row: 0 })));
        assert_eq!(rows.next(), Some(Ok((&1, &"b".to_string()))));
        assert!(rows.next().is_none());
    }

    #[test]
    fn rows_should_return_error_if_a_row_has_a_cell_with_wrong_type() {
        let mut table = T2::new();
        table.push_row((0, "a".to_string()));
        table.push_row((1, "b".to_string()));

        // Change "a" of Cell2::B(...) into Cell2::A(...)
        table.0.insert_cell(0, 1, Cell2::A(0));

        let mut rows = table.rows();
        assert_eq!(rows.next(), Some(Err(Table2Error::MissingBCell { row: 0 })));
        assert_eq!(rows.next(), Some(Ok((&1, &"b".to_string()))));
        assert!(rows.next().is_none());
    }
}
