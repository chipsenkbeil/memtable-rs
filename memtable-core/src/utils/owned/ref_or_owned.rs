use core::{
    borrow::Borrow,
    cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd},
    convert::{AsRef, From},
    default::Default,
    fmt,
    hash::{Hash, Hasher},
    ops::Deref,
};
use paste::paste;

/// Represents a generic wrapper around some data that can provide immutable
/// access to borrowed data. Deref is also implemented for the underlying data.
///
/// This exists as a simplified version of [`std::borrow::Cow`] as clone-on-write
/// pointers are not available in `core`. When compiled with `std`, this data
/// can be converted into a `Cow<'a, T>` instance.
#[derive(Copy)]
pub enum RefOrOwned<'a, T: 'a> {
    Borrowed(&'a T),
    Owned(T),
}

impl<'a, T: 'a> RefOrOwned<'a, T> {
    /// Returns true if the data is borrowed
    pub fn is_borrowed(&self) -> bool {
        matches!(self, Self::Borrowed(_))
    }

    /// Returns true if the data is owned
    pub fn is_owned(&self) -> bool {
        matches!(self, Self::Owned(_))
    }

    /// Consumes the wrapper and attempts to return the borrowed version underneath
    pub fn into_borrowed(self) -> Option<&'a T> {
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
    pub fn map_either<F1, F2, R>(self, f1: F1, f2: F2) -> RefOrOwned<'a, R>
    where
        F1: FnOnce(&'a T) -> &'a R,
        F2: FnOnce(T) -> R,
    {
        match self {
            Self::Borrowed(x) => RefOrOwned::Borrowed(f1(x)),
            Self::Owned(x) => RefOrOwned::Owned(f2(x)),
        }
    }
}

impl<'a, T: 'a> From<&'a T> for RefOrOwned<'a, T> {
    fn from(x: &'a T) -> Self {
        Self::Borrowed(x)
    }
}

impl<T> From<T> for RefOrOwned<'_, T> {
    fn from(x: T) -> Self {
        Self::Owned(x)
    }
}

impl<'a, T: 'a> Deref for RefOrOwned<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match *self {
            Self::Borrowed(borrowed) => borrowed,
            Self::Owned(ref owned) => owned.borrow(),
        }
    }
}

impl<T> Eq for RefOrOwned<'_, T> where T: Eq {}

impl<'a, 'b, T1, T2> PartialEq<RefOrOwned<'b, T2>> for RefOrOwned<'a, T1>
where
    T1: PartialEq<T2>,
{
    #[inline]
    fn eq(&self, other: &RefOrOwned<'b, T2>) -> bool {
        PartialEq::eq(&**self, &**other)
    }
}

macro_rules! impl_peq {
    ($type:ty) => {
        paste! {
            impl_peq!(@no_test $type);

            #[test]
            fn [< eq_against_ $type>]() {
                let x: RefOrOwned<'_, $type> = RefOrOwned::Owned(Default::default());
                assert_eq!(x, $type::default());
            }
        }
    };
    (@no_test $type:ty) => {
        paste! {
            impl<'a, T> PartialEq<$type> for RefOrOwned<'a, T>
            where
                T: PartialEq<$type>,
            {
                #[inline]
                fn eq(&self, other: &$type) -> bool {
                    PartialEq::eq(&**self, &*other)
                }
            }

            impl<'a, T> PartialEq<Option<$type>> for RefOrOwned<'a, T>
            where
                T: PartialEq<$type>,
            {
                #[inline]
                fn eq(&self, other: &Option<$type>) -> bool {
                    if let Some(x) = other.as_ref() {
                        PartialEq::eq(&**self, x)
                    } else {
                        false
                    }
                }
            }

            impl<'a, T, E> PartialEq<Result<$type, E>> for RefOrOwned<'a, T>
            where
                T: PartialEq<$type>,
            {
                #[inline]
                fn eq(&self, other: &Result<$type, E>) -> bool {
                    if let Ok(x) = other.as_ref() {
                        PartialEq::eq(&**self, x)
                    } else {
                        false
                    }
                }
            }
        }
    };
}

impl_peq!(bool);
impl_peq!(char);
impl_peq!(u8);
impl_peq!(u16);
impl_peq!(u32);
impl_peq!(u64);
impl_peq!(usize);
impl_peq!(i8);
impl_peq!(i16);
impl_peq!(i32);
impl_peq!(i64);
impl_peq!(isize);
impl_peq!(f32);
impl_peq!(f64);

impl<T> Ord for RefOrOwned<'_, T>
where
    T: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&**self, &**other)
    }
}

impl<'a, T> PartialOrd for RefOrOwned<'a, T>
where
    T: PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &RefOrOwned<'a, T>) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
}

impl<T> fmt::Debug for RefOrOwned<'_, T>
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

impl<T> fmt::Display for RefOrOwned<'_, T>
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

impl<T> Default for RefOrOwned<'_, T>
where
    T: Default,
{
    fn default() -> Self {
        RefOrOwned::Owned(<T as Default>::default())
    }
}

impl<T> Hash for RefOrOwned<'_, T>
where
    T: Hash,
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&**self, state)
    }
}

impl<T> AsRef<T> for RefOrOwned<'_, T> {
    fn as_ref(&self) -> &T {
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

    impl_peq!(@no_test &'a str);

    #[test]
    fn eq_against_str() {
        let x = RefOrOwned::Owned("some str");
        assert_eq!(x, "some str");
    }

    #[cfg(feature = "std")]
    impl_peq!(@no_test &'a std::path::Path);
    #[cfg(feature = "std")]
    impl_peq!(@no_test &'a std::ffi::OsStr);

    impl<'a, T> RefOrOwned<'a, T>
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

    impl<T> Clone for RefOrOwned<'_, T>
    where
        T: Clone,
    {
        fn clone(&self) -> Self {
            match *self {
                Self::Borrowed(b) => Self::Borrowed(b),
                Self::Owned(ref o) => {
                    let x: &T = o.borrow();
                    Self::Owned(x.to_owned())
                }
            }
        }
    }
}
