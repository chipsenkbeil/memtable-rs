use super::{utils, TableColumn};
use syn::{parse_quote, ItemConst};

pub struct Args<'a> {
    pub columns: &'a [&'a TableColumn],
}

pub fn make(args: Args) -> ItemConst {
    let Args { columns } = args;

    let column_names = utils::make_column_names(columns, ToString::to_string);

    parse_quote! {
        /// Represents the names of the columns associated with this type of table
        const COLUMN_NAMES: &'static [&'static ::core::primitive::str] = &[#(#column_names),*];
    }
}
