mod make;

pub use make::array::{default_array, make_array, try_make_array};
pub use make::table_array::{default_table_array, make_table_array, try_make_table_array};

#[cfg(feature = "serde-1")]
mod ser;

#[cfg(feature = "serde-1")]
#[doc(inline)]
#[cfg_attr(feature = "docs", doc(cfg(any(alloc, std))))]
pub use ser::{serialize_array, serialize_table_array, serialize_vec_array};

#[cfg(feature = "serde-1")]
mod de;

#[cfg(feature = "serde-1")]
#[doc(inline)]
#[cfg_attr(feature = "docs", doc(cfg(any(alloc, std))))]
pub use de::{deserialize_array, deserialize_table_array, deserialize_vec_array};
