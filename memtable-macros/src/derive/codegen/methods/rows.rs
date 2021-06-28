use super::{utils, TableColumn, TableMode};
use syn::{parse_quote, ItemFn, Path, Type};

pub struct Args<'a> {
    pub root: &'a Path,
    pub mode: TableMode,
    pub columns: &'a [&'a TableColumn],
}

pub fn make(args: Args) -> ItemFn {
    let Args {
        root,
        mode,
        columns,
    } = args;

    let variant_tys = utils::make_variant_types(columns);
    let bug_msg = utils::bug_str();

    // (type1, type2, ...)
    let iter_item_ty: Type = {
        match mode {
            TableMode::Ref => parse_quote!((#(&#variant_tys),*)),
            TableMode::Owned => parse_quote!((#(#variant_tys),*)),
            TableMode::Mixed => parse_quote!((#(#root::RefOrOwned<'_, #variant_tys>),*)),
        }
    };

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
