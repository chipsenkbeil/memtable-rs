use super::{utils, TableColumn};
use syn::{parse_quote, ItemFn, Path, Type};

pub struct Args<'a> {
    pub root: &'a Path,
    pub columns: &'a [&'a TableColumn],
}

pub fn make(args: Args) -> ItemFn {
    let Args { root, columns } = args;

    let variant_tys = utils::make_variant_types(columns);

    // (type1, type2, ...)
    let iter_item_ty: Type = parse_quote!((#(&#variant_tys),*));
    let bug_msg = utils::bug_str();

    parse_quote! {
        /// Iterates through each row of the table, returning a tuple of references
        /// to the individual fields
        pub fn rows(&self) -> impl ::std::iter::Iterator<Item = #iter_item_ty> {
            // NOTE: The expect(...) should never happen as we should have
            //       all of the rows available in the described range
            ::std::iter::Iterator::map(
                0..#root::Table::row_cnt(&self.0),
                move |idx| self.row(idx).expect(#bug_msg),
            )
        }
    }
}
