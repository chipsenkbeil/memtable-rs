use super::{utils, TableColumn};
use syn::{parse_quote, Generics, Ident, ItemFn, Path};

pub struct Args<'a> {
    pub root: &'a Path,
    pub generics: &'a Generics,
    pub columns: &'a [&'a TableColumn],
    pub origin_struct_name: &'a Ident,
    pub table_data_name: &'a Ident,
}

pub fn make(args: Args) -> ItemFn {
    let Args {
        root,
        generics,
        columns,
        origin_struct_name,
        table_data_name,
    } = args;

    let (_, ty_generics, _) = generics.split_for_impl();
    let fields = utils::make_field_tokens(columns);
    let variants = utils::make_variant_idents(columns);

    parse_quote! {
        /// Inserts a new row into the table at the given position, shifting down
        /// all rows after it
        pub fn insert_row<__RowData: ::core::convert::Into<#origin_struct_name #ty_generics>>(
            &mut self,
            row: ::core::primitive::usize,
            data: __RowData,
        ) {
            let data = data.into();
            #root::Table::insert_row(
                &mut self.0,
                row,
                ::core::array::IntoIter::new([
                    #(#table_data_name::#variants(data.#fields)),*
                ])
            );
        }
    }
}
