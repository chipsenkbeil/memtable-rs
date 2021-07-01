use syn::{parse_quote, Generics, Ident, ItemFn, Path};

pub struct Args<'a> {
    pub root: &'a Path,
    pub generics: &'a Generics,
    pub origin_struct_name: &'a Ident,
}

pub fn make(args: Args) -> ItemFn {
    let Args {
        root,
        generics,
        origin_struct_name,
    } = args;

    let (_, ty_generics, _) = generics.split_for_impl();

    parse_quote! {
        /// Pushes a row to the end of the table
        pub fn push_row<__RowData: ::core::convert::Into<#origin_struct_name #ty_generics>>(
            &mut self,
            data: __RowData,
        ) {
            self.insert_row(#root::Table::row_cnt(&self.0), data)
        }
    }
}
