use super::{utils, TableColumn};
use syn::{parse_quote, ItemFn};

pub struct Args<'a> {
    pub columns: &'a [&'a TableColumn],
}

pub fn make(args: Args) -> ItemFn {
    let Args { columns } = args;

    let column_names = utils::make_column_names(columns, ToString::to_string);

    parse_quote! {
        /// Returns the numbers of the columns associated with this type of table
        pub const fn column_names() -> &'static [&'static ::std::primitive::str] {
            &[#(#column_names),*]
        }
    }
}
