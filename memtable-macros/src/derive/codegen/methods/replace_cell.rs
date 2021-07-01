use syn::{parse_quote, Ident, ItemFn, Path, Type};

pub struct Args<'a> {
    pub root: &'a Path,
    pub idx: &'a syn::Index,
    pub method_name: &'a Ident,
    pub variant_ty: &'a Type,
    pub table_data_name: &'a Ident,
    pub variant: &'a Ident,
    pub into_variant: &'a Ident,
}

pub fn make(args: Args) -> ItemFn {
    let Args {
        root,
        idx,
        method_name,
        variant_ty,
        table_data_name,
        variant,
        into_variant,
    } = args;

    parse_quote! {
        /// Swaps the current cell value with the provided one, doing nothing
        /// if there is no cell at the specified row for the explicit column
        pub fn #method_name<__Value: ::core::convert::Into<#variant_ty>>(
            &mut self,
            row: ::core::primitive::usize,
            value: __Value,
        ) -> ::core::option::Option<#variant_ty> {
            if row < #root::Table::row_cnt(&self.0) {
                #root::Table::insert_cell(
                    &mut self.0,
                    row,
                    #idx,
                    #table_data_name::#variant(value.into()),
                ).and_then(#table_data_name::#into_variant)
            } else {
                ::core::option::Option::None
            }
        }
    }
}
