use crate::{
    iter::{Row, Rows},
    Table,
};
use predicates::Predicate;
use std::fmt;

pub struct FilterRows<'a, D, T, P>
where
    T: Table<Data = D> + 'a,
    P: Predicate<Row<'a, D, T>>,
{
    rows: Rows<'a, D, T>,
    predicate: P,
}

impl<'a, D, T, P> FilterRows<'a, D, T, P>
where
    T: Table<Data = D> + 'a,
    P: Predicate<Row<'a, D, T>>,
{
    pub fn new(rows: Rows<'a, D, T>, predicate: P) -> Self {
        Self { rows, predicate }
    }
}

impl<'a, D, T, P> Iterator for FilterRows<'a, D, T, P>
where
    T: Table<Data = D> + 'a,
    P: Predicate<Row<'a, D, T>>,
{
    type Item = Row<'a, D, T>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(row) = self.rows.next() {
            if self.predicate.eval(&row) {
                return Some(row);
            }
        }

        None
    }
}

impl<'a, D, T, P> fmt::Debug for FilterRows<'a, D, T, P>
where
    D: fmt::Debug,
    T: Table<Data = D> + fmt::Debug + 'a,
    P: Predicate<Row<'a, D, T>>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FilterRows")
            .field("rows", &self.rows)
            .finish_non_exhaustive()
    }
}
