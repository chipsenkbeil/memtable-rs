use super::{utils, TableColumn};
use syn::{parse_quote, Ident, ItemFn, Path, Type};

pub struct Args<'a> {
    pub root: &'a Path,
    pub columns: &'a [&'a TableColumn],
}

pub fn make(args: Args) -> ItemFn {
    let Args { root, columns } = args;

    let variant_tys = utils::make_variant_types(columns);
    let cell_fns: Vec<Ident> = utils::make_snake_idents(columns);

    // (type1, type2, ...)
    let option_inner_ty: Type = parse_quote!((#(&#variant_tys),*));

    let bug_msg = utils::bug_str();

    parse_quote! {
        /// Returns a tuple containing refs to each column's data within a row
        pub fn row(
            &self,
            row: ::core::primitive::usize,
        ) -> ::core::option::Option<#option_inner_ty> {
            // NOTE: Because we don't allow access to the underlying table
            //       at the level where the cell enum can be changed to
            //       another type, this should NEVER fail. We want to rely
            //       on that guarantee as it would be considered corrupt
            //       if the data changed types underneath.
            if row < #root::Table::row_cnt(&self.0) {
                ::core::option::Option::Some(
                    (#(
                        self.#cell_fns(row).expect(#bug_msg)
                    ),*)
                )
            } else {
                ::core::option::Option::None
            }
        }
    }
}
