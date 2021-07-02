use core::fmt;
use predicates::Predicate;

/// Represents a filter over an iterator based on a provided predicate
pub struct FilterByPredicate<T, I, P>
where
    I: Iterator<Item = T>,
    P: Predicate<T>,
{
    iter: I,
    predicate: P,
}

impl<T, I, P> FilterByPredicate<T, I, P>
where
    I: Iterator<Item = T>,
    P: Predicate<T>,
{
    pub fn new(iter: I, predicate: P) -> Self {
        Self { iter, predicate }
    }
}

impl<T, I, P> Iterator for FilterByPredicate<T, I, P>
where
    I: Iterator<Item = T>,
    P: Predicate<T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.iter.next() {
            if self.predicate.eval(&item) {
                return Some(item);
            }
        }

        None
    }
}

impl<T, I, P> fmt::Debug for FilterByPredicate<T, I, P>
where
    T: fmt::Debug,
    I: Iterator<Item = T> + fmt::Debug,
    P: Predicate<T>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FilterByPredicate")
            .field("iter", &self.iter)
            .finish_non_exhaustive()
    }
}
