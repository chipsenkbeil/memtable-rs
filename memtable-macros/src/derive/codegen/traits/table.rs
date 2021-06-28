use syn::{parse_quote, Generics, Ident, ItemImpl, Path};

pub struct Args<'a> {
    pub root: &'a Path,
    pub table_name: &'a Ident,
    pub generics: &'a Generics,
    pub table_data_name: &'a Ident,
}

pub fn make(args: Args) -> ItemImpl {
    let Args {
        root,
        table_name,
        generics,
        table_data_name,
    } = args;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    parse_quote! {
        impl #impl_generics #root::Table for #table_name #ty_generics #where_clause {
            type Data = #table_data_name #ty_generics;

            fn row_cnt(&self) -> ::std::primitive::usize {
                #root::Table::row_cnt(&self.0)
            }

            fn col_cnt(&self) -> ::std::primitive::usize {
                #root::Table::col_cnt(&self.0)
            }

            fn get_cell(
                &self,
                row: ::std::primitive::usize,
                col: ::std::primitive::usize,
            ) -> ::std::option::Option<#root::RefOrOwned<'_, Self::Data>> {
                #root::Table::get_cell(&self.0, row, col)
            }

            fn get_mut_cell(
                &mut self,
                row: ::std::primitive::usize,
                col: ::std::primitive::usize,
            ) -> ::std::option::Option<#root::MutRefOrOwned<'_, Self::Data>> {
                #root::Table::get_mut_cell(&mut self.0, row, col)
            }

            fn insert_cell(
                &mut self,
                row: ::std::primitive::usize,
                col: ::std::primitive::usize,
                value: Self::Data,
            ) -> ::std::option::Option<Self::Data> {
                #root::Table::insert_cell(&mut self.0, row, col, value)
            }

            fn remove_cell(
                &mut self,
                row: ::std::primitive::usize,
                col: ::std::primitive::usize,
            ) -> ::std::option::Option<Self::Data> {
                #root::Table::remove_cell(&mut self.0, row, col)
            }

            fn set_row_capacity(&mut self, capacity: ::std::primitive::usize) {
                #root::Table::set_row_capacity(&mut self.0, capacity);
            }

            fn set_column_capacity(&mut self, capacity: ::std::primitive::usize) {
                #root::Table::set_column_capacity(&mut self.0, capacity);
            }
        }
    }
}
