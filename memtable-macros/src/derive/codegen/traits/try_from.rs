use super::{utils, TableColumn, TableMode};
use quote::format_ident;
use syn::{parse_quote, Generics, Ident, ItemImpl, Path};
use voca_rs::case;

pub struct Args<'a> {
    pub root: &'a Path,
    pub mode: TableMode,
    pub table_name: &'a Ident,
    pub generics: &'a Generics,
    pub table_data_name: &'a Ident,
    pub columns: &'a [&'a TableColumn],
}

pub fn make(args: Args) -> ItemImpl {
    let Args {
        root,
        mode,
        table_name,
        generics,
        table_data_name,
        columns,
    } = args;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let variant = utils::make_variant_idents(columns);
    let ty = utils::make_variant_types(columns);
    let is_ty: Vec<Ident> = utils::make_column_names(columns, case::snake_case)
        .into_iter()
        .map(|name| format_ident!("is_{}", name))
        .collect();
    let idx = utils::make_column_indexes(columns);
    let inner_table_ty =
        utils::make_inner_table_type(root, mode, table_data_name, generics, columns.len());

    parse_quote! {
        #[automatically_derived]
        impl #impl_generics ::core::convert::TryFrom<#inner_table_ty>
            for #table_name #ty_generics #where_clause
        {
            type Error = &'static ::core::primitive::str;

            fn try_from(table: #inner_table_ty) -> ::core::result::Result<Self, Self::Error> {
                for row in 0..#root::Table::row_cnt(&table) {
                    #(
                        let cell = #root::Table::get_cell(&table, row, #idx);

                        if cell.is_none() {
                            return ::core::result::Result::Err(
                                ::core::concat!(
                                    "Cell in column ",
                                    ::core::stringify!(#idx),
                                    "/",
                                    ::core::stringify!(#variant),
                                    " is missing",
                                )
                            );
                        }

                        if !cell.unwrap().#is_ty() {
                            return ::core::result::Result::Err(
                                ::core::concat!(
                                    "Cell in column ",
                                    ::core::stringify!(#idx),
                                    "/",
                                    ::core::stringify!(#variant),
                                    " is not of type ",
                                    ::core::stringify!(#ty),
                                )
                            );
                        }
                    )*
                }

                ::core::result::Result::Ok(Self(table))
            }
        }
    }
}
