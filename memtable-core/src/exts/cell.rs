use paste::paste;

macro_rules! impl_cell {
    ($name:ident $($variant:ident)+) => {
        paste! {
            /// Represents a data stucture that can hold a variety of different data types
            #[cfg_attr(feature = "docs", doc(cfg(cell)))]
            #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
            #[cfg_attr(feature = "serde-1", derive(serde::Serialize, serde::Deserialize))]
            pub enum $name<$($variant),+> {
                $(
                    #[doc = "Represents container for `" $variant "` generic type"]
                    $variant($variant)
                ),+
            }

            impl<$($variant),+> $name<$($variant),+> {
                $(
                    #[doc = "Returns true if `" $name "` is the `" $variant "` variant"]
                    pub fn [<is_ $variant:snake>](&self) -> bool {
                        matches!(self, Self::$variant(_))
                    }

                    #[doc = "Attempts to cast to a reference of the `" $variant "` variant"]
                    pub fn [<as_ $variant:snake>](&self) -> Option<&$variant> {
                        match self {
                            Self::$variant(x) => Some(x),
                            _ => None,
                        }
                    }

                    #[doc = "Attempts to cast to a mutable reference of the `" $variant "` variant"]
                    pub fn [<as_mut_ $variant:snake>](&mut self) -> Option<&mut $variant> {
                        match self {
                            Self::$variant(x) => Some(x),
                            _ => None,
                        }
                    }

                    #[doc = "Attempts to consume and convert into the `" $variant "` variant"]
                    pub fn [<into_ $variant:snake>](self) -> Option<$variant> {
                        match self {
                            Self::$variant(x) => Some(x),
                            _ => None,
                        }
                    }
                )+
            }
        }
    };
}

impl_cell!(Cell2 A B);
impl_cell!(Cell3 A B C);
impl_cell!(Cell4 A B C D);
impl_cell!(Cell5 A B C D E);
impl_cell!(Cell6 A B C D E F);
impl_cell!(Cell7 A B C D E F G);
impl_cell!(Cell8 A B C D E F G H);
impl_cell!(Cell9 A B C D E F G H I);
impl_cell!(Cell10 A B C D E F G H I J);
impl_cell!(Cell11 A B C D E F G H I J K);
impl_cell!(Cell12 A B C D E F G H I J K L);
impl_cell!(Cell13 A B C D E F G H I J K L M);
impl_cell!(Cell14 A B C D E F G H I J K L M N);
impl_cell!(Cell15 A B C D E F G H I J K L M N O);
impl_cell!(Cell16 A B C D E F G H I J K L M N O P);
impl_cell!(Cell17 A B C D E F G H I J K L M N O P Q);
impl_cell!(Cell18 A B C D E F G H I J K L M N O P Q R);
impl_cell!(Cell19 A B C D E F G H I J K L M N O P Q R S);
impl_cell!(Cell20 A B C D E F G H I J K L M N O P Q R S T);
impl_cell!(Cell21 A B C D E F G H I J K L M N O P Q R S T U);
impl_cell!(Cell22 A B C D E F G H I J K L M N O P Q R S T U V);
impl_cell!(Cell23 A B C D E F G H I J K L M N O P Q R S T U V W);
impl_cell!(Cell24 A B C D E F G H I J K L M N O P Q R S T U V W X);
impl_cell!(Cell25 A B C D E F G H I J K L M N O P Q R S T U V W X Y);
impl_cell!(Cell26 A B C D E F G H I J K L M N O P Q R S T U V W X Y Z);
