use super::{utils, TableColumn};
use syn::{parse_quote, Generics, Ident, ItemFn, Path};

pub struct Args<'a> {
    pub root: &'a Path,
    pub generics: &'a Generics,
    pub table_data_name: &'a Ident,
    pub columns: &'a [&'a TableColumn],
}

pub fn make(args: Args) -> ItemFn {
    let Args {
        root,
        generics,
        table_data_name,
        columns,
    } = args;

    let (_, ty_generics, _) = generics.split_for_impl();
    let column_names = utils::make_column_names(columns, ToString::to_string);
    let idx = utils::make_column_indexes(columns);

    parse_quote! {
        /// Retrieves a column by its name
        pub fn column_by_name(
            &self,
            name: &::std::primitive::str,
        ) -> ::std::option::Option<#root::iter::Column<
            #table_data_name #ty_generics,
            #root::MemDynamicTable<#table_data_name #ty_generics>,
        >> {
            match name {
                #(
                    #column_names => ::std::option::Option::Some(
                        #root::Table::column(&self.0, #idx)
                    ),
                )*
                _ => ::std::option::Option::None,
            }
        }
    }
}
