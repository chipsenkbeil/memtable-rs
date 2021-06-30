use super::{utils, TableColumn};
use darling::ast::Style;
use quote::format_ident;
use syn::{parse_quote, Ident, ItemFn, Path, Type};

pub struct Args<'a> {
    pub root: &'a Path,
    pub style: Style,
    pub columns: &'a [&'a TableColumn],
}

pub fn make(args: Args) -> ItemFn {
    let Args {
        root,
        style,
        columns,
    } = args;

    let variant_tys = utils::make_variant_types(columns);
    let get_cell_fns: Vec<Ident> = utils::make_snake_idents(columns)
        .iter()
        .map(|name| {
            format_ident!(
                "get_cell{}{}",
                if style.is_tuple() { "" } else { "_" },
                name
            )
        })
        .collect();

    // (type1, type2, ...)
    let option_inner_ty: Type = parse_quote!((#(&#variant_tys),*));

    let bug_msg = utils::bug_str();

    parse_quote! {
        /// Returns a tuple containing refs to each column's data within a row
        pub fn row(
            &self,
            row: ::std::primitive::usize,
        ) -> ::std::option::Option<#option_inner_ty> {
            // NOTE: Because we don't allow access to the underlying table
            //       at the level where the cell enum can be changed to
            //       another type, this should NEVER fail. We want to rely
            //       on that guarantee as it would be considered corrupt
            //       if the data changed types underneath.
            if row < #root::Table::row_cnt(&self.0) {
                ::std::option::Option::Some(
                    (#(
                        self.#get_cell_fns(row).expect(#bug_msg)
                    ),*)
                )
            } else {
                ::std::option::Option::None
            }
        }
    }
}
