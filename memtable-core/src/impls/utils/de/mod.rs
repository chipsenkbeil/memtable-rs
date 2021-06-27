mod array;
mod table_array;
mod vec_array;

use super::{default_array, default_table_array};
pub use array::deserialize_array;
pub use table_array::deserialize_table_array;
pub use vec_array::deserialize_vec_array;
