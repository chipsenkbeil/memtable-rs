use core::{
    borrow::Borrow,
    cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd},
    convert::{AsMut, AsRef, From},
    default::Default,
    fmt,
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
};

/// Represents a generic wrapper around some data that can provide mutable
/// access to borrowed data. DerefMut is also implemented for the underlying data.
///
/// This exists as a simplified version of [`std::borrow::Cow`] as clone-on-write
/// pointers are not available in `core`. When compiled with `std`, this data
/// can be converted into a `Cow<'a, T>` instance by cloning the data regardless.
pub enum MutRefOrOwned<'a, T: 'a> {
    Borrowed(&'a mut T),
    Owned(T),
}

impl<'a, T: 'a> MutRefOrOwned<'a, T> {
    /// Returns true if the data is borrowed
    pub fn is_borrowed(&self) -> bool {
        matches!(self, Self::Borrowed(_))
    }

    /// Returns true if the data is owned
    pub fn is_owned(&self) -> bool {
        matches!(self, Self::Owned(_))
    }

    /// Consumes the wrapper and attempts to return the borrowed version underneath
    pub fn into_borrowed(self) -> Option<&'a mut T> {
        match self {
            Self::Borrowed(x) => Some(x),
            _ => None,
        }
    }

    /// Consumes the wrapper and attempts to return the owned version underneath
    ///
    /// Unlike [`std::borrow::Cow`], this does not attempt to clone the data if
    /// it is a reference; so, in the event that the data is a reference, the
    /// underlying reference will be dropped
    pub fn into_owned(self) -> Option<T> {
        match self {
            Self::Owned(x) => Some(x),
            _ => None,
        }
    }

    /// Consumes and maps the wrapper's contents into a new form
    pub fn map_either<F1, F2, R>(self, f1: F1, f2: F2) -> MutRefOrOwned<'a, R>
    where
        F1: FnOnce(&'a mut T) -> &'a mut R,
        F2: FnOnce(T) -> R,
    {
        match self {
            Self::Borrowed(x) => MutRefOrOwned::Borrowed(f1(x)),
            Self::Owned(x) => MutRefOrOwned::Owned(f2(x)),
        }
    }
}

impl<'a, T: 'a> From<&'a mut T> for MutRefOrOwned<'a, T> {
    fn from(x: &'a mut T) -> Self {
        Self::Borrowed(x)
    }
}

impl<T> From<T> for MutRefOrOwned<'_, T> {
    fn from(x: T) -> Self {
        Self::Owned(x)
    }
}

impl<'a, T: 'a> Deref for MutRefOrOwned<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match *self {
            Self::Borrowed(ref borrowed) => borrowed,
            Self::Owned(ref owned) => owned.borrow(),
        }
    }
}

impl<'a, T: 'a> DerefMut for MutRefOrOwned<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match *self {
            Self::Borrowed(ref mut borrowed) => borrowed,
            Self::Owned(ref mut owned) => owned,
        }
    }
}

impl<T> Eq for MutRefOrOwned<'_, T> where T: Eq {}

impl<'a, T> PartialEq<&'a T> for MutRefOrOwned<'a, T>
where
    T: PartialEq,
{
    #[inline]
    fn eq(&self, other: &&'a T) -> bool {
        PartialEq::eq(self.as_ref(), other)
    }
}

impl<'a, 'b, T1, T2> PartialEq<MutRefOrOwned<'b, T2>> for MutRefOrOwned<'a, T1>
where
    T1: PartialEq<T2>,
{
    #[inline]
    fn eq(&self, other: &MutRefOrOwned<'b, T2>) -> bool {
        PartialEq::eq(&**self, &**other)
    }
}

impl<T> Ord for MutRefOrOwned<'_, T>
where
    T: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&**self, &**other)
    }
}

impl<'a, T> PartialOrd for MutRefOrOwned<'a, T>
where
    T: PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &MutRefOrOwned<'a, T>) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
}

impl<T> fmt::Debug for MutRefOrOwned<'_, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Borrowed(ref b) => fmt::Debug::fmt(b, f),
            Self::Owned(ref o) => fmt::Debug::fmt(o, f),
        }
    }
}

impl<T> fmt::Display for MutRefOrOwned<'_, T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Borrowed(ref b) => fmt::Display::fmt(b, f),
            Self::Owned(ref o) => fmt::Display::fmt(o, f),
        }
    }
}

impl<T> Default for MutRefOrOwned<'_, T>
where
    T: Default,
{
    fn default() -> Self {
        MutRefOrOwned::Owned(<T as Default>::default())
    }
}

impl<T> Hash for MutRefOrOwned<'_, T>
where
    T: Hash,
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&**self, state)
    }
}

impl<T> AsRef<T> for MutRefOrOwned<'_, T> {
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T> AsMut<T> for MutRefOrOwned<'_, T> {
    fn as_mut(&mut self) -> &mut T {
        self
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
mod alloc_or_std {
    use super::*;

    #[cfg(feature = "std")]
    use std::{borrow::Cow, clone::Clone};

    #[cfg(not(feature = "std"))]
    use alloc::{borrow::Cow, clone::Clone};

    #[cfg_attr(feature = "docs", doc(cfg(any(feature = "alloc", feature = "std"))))]
    impl<'a, T> MutRefOrOwned<'a, T>
    where
        T: Clone,
    {
        pub fn into_cow(self) -> Cow<'a, T> {
            match self {
                Self::Borrowed(x) => Cow::Borrowed(x),
                Self::Owned(x) => Cow::Owned(x),
            }
        }
    }

    #[cfg_attr(feature = "docs", doc(cfg(any(feature = "alloc", feature = "std"))))]
    impl<T> Clone for MutRefOrOwned<'_, T>
    where
        T: Clone,
    {
        /// Implementation will always yield a new, owned instance as we
        /// cannot copy a `&mut` reference
        fn clone(&self) -> Self {
            Self::Owned(self.as_ref().to_owned())
        }
    }
}
