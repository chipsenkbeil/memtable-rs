use super::{TableColumn, TableMode};
use syn::{parse_quote, Generics, Ident, ItemImpl, Path, Type};

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
    let cols = columns.len();

    let row_t: Type = match mode {
        TableMode::Dynamic => {
            parse_quote!(#root::list::DynamicList<Self::Data>)
        }
        TableMode::Fixed { .. } | TableMode::FixedColumn => {
            parse_quote!(#root::list::FixedList<Self::Data, #cols>)
        }
    };
    let col_t: Type = match mode {
        TableMode::Dynamic | TableMode::FixedColumn => {
            parse_quote!(#root::list::DynamicList<Self::Data>)
        }
        TableMode::Fixed { rows } => parse_quote!(#root::list::FixedList<Self::Data, #rows>),
    };

    parse_quote! {
        impl #impl_generics #root::Table for #table_name #ty_generics #where_clause {
            type Data = #table_data_name #ty_generics;
            type Row = #row_t;
            type Column = #col_t;

            fn row_cnt(&self) -> ::core::primitive::usize {
                #root::Table::row_cnt(&self.0)
            }

            fn col_cnt(&self) -> ::core::primitive::usize {
                #root::Table::col_cnt(&self.0)
            }

            fn get_cell(
                &self,
                row: ::core::primitive::usize,
                col: ::core::primitive::usize,
            ) -> ::core::option::Option<&Self::Data> {
                #root::Table::get_cell(&self.0, row, col)
            }

            fn get_mut_cell(
                &mut self,
                row: ::core::primitive::usize,
                col: ::core::primitive::usize,
            ) -> ::core::option::Option<&mut Self::Data> {
                #root::Table::get_mut_cell(&mut self.0, row, col)
            }

            fn insert_cell(
                &mut self,
                row: ::core::primitive::usize,
                col: ::core::primitive::usize,
                value: Self::Data,
            ) -> ::core::option::Option<Self::Data> {
                #root::Table::insert_cell(&mut self.0, row, col, value)
            }

            fn remove_cell(
                &mut self,
                row: ::core::primitive::usize,
                col: ::core::primitive::usize,
            ) -> ::core::option::Option<Self::Data> {
                #root::Table::remove_cell(&mut self.0, row, col)
            }

            fn set_row_capacity(&mut self, capacity: ::core::primitive::usize) {
                #root::Table::set_row_capacity(&mut self.0, capacity);
            }

            fn set_column_capacity(&mut self, capacity: ::core::primitive::usize) {
                #root::Table::set_column_capacity(&mut self.0, capacity);
            }
        }
    }
}
