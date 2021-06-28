use super::{utils, TableColumn, TableMode};
use quote::format_ident;
use syn::{parse_quote, Ident, ItemFn, Path, Type};

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
    let get_cell_fns: Vec<Ident> = utils::make_snake_idents(columns)
        .iter()
        .map(|name| format_ident!("get_{}", name))
        .collect();

    let bug_msg = utils::bug_str();

    // (type1, type2, ...)
    let option_inner_ty: Type = {
        match mode {
            TableMode::Ref => parse_quote!((#(&#variant_tys),*)),
            TableMode::Owned => parse_quote!((#(#variant_tys),*)),
            TableMode::Mixed => parse_quote!((#(#root::RefOrOwned<'_, #variant_tys>),*)),
        }
    };

    let doc = format!(
        "Returns a tuple containing {}",
        match mode {
            TableMode::Ref => "refs to each column's data within a row",
            TableMode::Owned => "each column's data within a row",
            TableMode::Mixed => "some mixture of refs & owned data for each column within a row",
        }
    );

    parse_quote! {
        #[doc = #doc]
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
