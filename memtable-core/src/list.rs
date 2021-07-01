use crate::utils;
use core::{
    array,
    cmp::Ordering,
    iter::FromIterator,
    mem,
    ops::{Deref, DerefMut},
};

/// Represents the capacity of the list
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub enum Capacity {
    Limited(usize),
    Unlimited,
}

impl Capacity {
    /// Returns true if the capacity is unlimited
    pub fn is_unlimited(self) -> bool {
        matches!(self, Self::Unlimited)
    }

    /// Returns true if the capacity is limited
    pub fn is_limited(self) -> bool {
        matches!(self, Self::Limited(_))
    }

    /// Returns the limit associated with the capacity if it has one
    pub fn limit(self) -> Option<usize> {
        match self {
            Self::Limited(x) => Some(x),
            _ => None,
        }
    }
}

/// Represents a generic list of items
pub trait List: Sized {
    type Item;

    /// Creates a new list with **up to** N elements, each created using
    /// the provided function; when the provided function returns None, the
    /// underlying list implementation will determine what to do
    fn new_filled_with<F: FnMut(usize) -> Option<Self::Item>>(n: usize, f: F) -> Self;

    /// Returns the maximum capacity of the list
    fn max_capacity(&self) -> Capacity;

    /// Returns the actual length of the list, which may be less than the
    /// actual capacity
    fn len(&self) -> usize;

    /// Returns true if the list is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a reference to an element found at the given index, or None
    /// if no element found
    fn get(&self, index: usize) -> Option<&Self::Item>;

    /// Returns a mutable reference to an element found at the given index, or
    /// None if no element found
    fn get_mut(&mut self, index: usize) -> Option<&mut Self::Item>;

    /// Inserts an element at position `index` within the vector, shifting all
    /// elements after it to the right
    ///
    /// # Panics
    ///
    /// Panics if `index > len`
    fn insert(&mut self, index: usize, element: Self::Item);

    /// Removes and returns the element at position `index` within the vector,
    /// shifting all elements after it to the left.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds
    fn remove(&mut self, index: usize) -> Self::Item;
}

/// Represents a fixed list that can grow up to a specific capacity `N`
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
pub struct FixedList<T: Default, const N: usize>(
    #[cfg_attr(
        feature = "serde-1",
        serde(
            bound(
                serialize = "T: serde::Serialize",
                deserialize = "T: serde::Deserialize<'de>"
            ),
            serialize_with = "utils::serialize_array",
            deserialize_with = "utils::deserialize_array"
        )
    )]
    [T; N],
    usize,
);

impl<T: Default, const N: usize> List for FixedList<T, N> {
    type Item = T;

    /// Will make a list that fills up to N, and any additional elements
    /// not filled in using the function will be completed with the default
    /// value; when the function returns None, the default value will also
    /// be used
    fn new_filled_with<F: FnMut(usize) -> Option<Self::Item>>(n: usize, mut f: F) -> Self {
        let arr = utils::make_array(|i| match f(i) {
            Some(data) if i < n => data,
            _ => T::default(),
        });

        Self(arr, N)
    }

    fn max_capacity(&self) -> Capacity {
        Capacity::Limited(N)
    }

    fn len(&self) -> usize {
        self.1
    }

    fn get(&self, index: usize) -> Option<&Self::Item> {
        self.0.get(index)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut Self::Item> {
        self.0.get_mut(index)
    }

    fn insert(&mut self, index: usize, element: Self::Item) {
        #[cold]
        #[inline(never)]
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!(
                "insertion index (is {}) should be <= len (is {})",
                index, len
            );
        }

        let len = self.len();
        if index > len {
            assert_failed(index, len);
        }

        self.0[index] = element;

        // space for the new element
        if index == len {
            self.1 += 1;
        }
    }

    fn remove(&mut self, index: usize) -> Self::Item {
        #[cold]
        #[inline(never)]
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("removal index (is {}) should be < len (is {})", index, len);
        }

        let len = self.len();
        if index >= len {
            assert_failed(index, len);
        }

        // First, remove the element by filling in with default value
        let data = mem::take(&mut self.0[index]);

        // Second, shift over all other elements
        for i in index + 1..len {
            let value = mem::take(&mut self.0[i]);
            self.0[i - 1] = value;
        }

        data
    }
}

impl<T: Default, const N: usize> Deref for FixedList<T, N> {
    type Target = [T; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Default, const N: usize> DerefMut for FixedList<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Default, const N: usize> From<FixedList<T, N>> for [T; N] {
    fn from(list: FixedList<T, N>) -> Self {
        list.0
    }
}

impl<T, U, const N: usize> PartialEq<[U; N]> for FixedList<T, N>
where
    T: PartialEq<U> + Default,
{
    fn eq(&self, other: &[U; N]) -> bool {
        PartialEq::eq(&self.0, &*other)
    }
}

impl<T, const N: usize> PartialOrd<[T; N]> for FixedList<T, N>
where
    T: PartialOrd<T> + Default,
{
    fn partial_cmp(&self, other: &[T; N]) -> Option<Ordering> {
        PartialOrd::partial_cmp(&self.0, &*other)
    }
}

impl<T: Default, const N: usize> IntoIterator for FixedList<T, N> {
    type Item = T;
    type IntoIter = array::IntoIter<Self::Item, N>;

    fn into_iter(self) -> Self::IntoIter {
        array::IntoIter::new(self.0)
    }
}

#[doc(inline)]
pub use self::alloc::DynamicList;

#[cfg(any(feature = "alloc", feature = "std"))]
mod alloc {
    use super::*;
    use std::vec::Vec;

    /// Represents a dynamic list that can grow and shrink with unlimited capacity
    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
    pub struct DynamicList<T>(Vec<T>);

    impl<T> List for DynamicList<T> {
        type Item = T;

        /// Makes a new list of **up to** N by using the provided function;
        /// whenever the function returns None, the item is skipped and the
        /// list's length will not grow
        fn new_filled_with<F: FnMut(usize) -> Option<Self::Item>>(n: usize, mut f: F) -> Self {
            let mut inner = Vec::new();

            for i in 0..n {
                if let Some(data) = f(i) {
                    inner.push(data);
                }
            }

            Self(inner)
        }

        fn max_capacity(&self) -> Capacity {
            Capacity::Unlimited
        }

        fn len(&self) -> usize {
            self.0.len()
        }

        fn get(&self, index: usize) -> Option<&Self::Item> {
            self.0.get(index)
        }

        fn get_mut(&mut self, index: usize) -> Option<&mut Self::Item> {
            self.0.get_mut(index)
        }

        fn insert(&mut self, index: usize, element: Self::Item) {
            self.0.insert(index, element)
        }

        fn remove(&mut self, index: usize) -> Self::Item {
            self.0.remove(index)
        }
    }

    impl<T> Deref for DynamicList<T> {
        type Target = Vec<T>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<T> DerefMut for DynamicList<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    impl<T> From<DynamicList<T>> for Vec<T> {
        fn from(list: DynamicList<T>) -> Self {
            list.0
        }
    }

    impl<T> From<Vec<T>> for DynamicList<T> {
        fn from(vec: Vec<T>) -> Self {
            Self(vec)
        }
    }

    impl<T, const N: usize> From<[T; N]> for DynamicList<T> {
        fn from(arr: [T; N]) -> Self {
            Self(Vec::from(arr))
        }
    }

    impl<T> FromIterator<T> for DynamicList<T> {
        fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
            Self(iter.into_iter().collect())
        }
    }

    impl<T, U> PartialEq<Vec<U>> for DynamicList<T>
    where
        T: PartialEq<U>,
    {
        fn eq(&self, other: &Vec<U>) -> bool {
            PartialEq::eq(&*self.0, &**other)
        }
    }

    impl<T, U, const N: usize> PartialEq<[U; N]> for DynamicList<T>
    where
        T: PartialEq<U>,
    {
        fn eq(&self, other: &[U; N]) -> bool {
            PartialEq::eq(&*self.0, &*other)
        }
    }

    impl<T> PartialOrd<Vec<T>> for DynamicList<T>
    where
        T: PartialOrd<T>,
    {
        fn partial_cmp(&self, other: &Vec<T>) -> Option<Ordering> {
            PartialOrd::partial_cmp(&*self.0, &**other)
        }
    }

    impl<T> IntoIterator for DynamicList<T> {
        type Item = T;
        type IntoIter = std::vec::IntoIter<Self::Item>;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }
}
