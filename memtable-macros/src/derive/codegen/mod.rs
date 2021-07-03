pub mod constants;
pub mod data;
pub mod methods;
pub mod parts;
pub mod traits;
pub mod utils;

use super::{TableColumn, TableMode};
use darling::ast::Style;
use syn::{parse_quote, Generics, Ident, ItemImpl, Path};

pub struct TableImplArgs<'a> {
    pub root: &'a Path,
    pub mode: TableMode,
    pub style: Style,
    pub origin_struct_name: &'a Ident,
    pub table_name: &'a Ident,
    pub generics: &'a Generics,
    pub table_data_name: &'a Ident,
    pub columns: &'a [&'a TableColumn],
}

pub fn make_table_impl(args: TableImplArgs) -> ItemImpl {
    let TableImplArgs {
        root,
        mode,
        style,
        origin_struct_name,
        table_name,
        generics,
        table_data_name,
        columns,
    } = args;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let column_names_const =
        constants::column_names::make(constants::column_names::Args { columns });
    let new_fn = methods::new::make(methods::new::Args {});
    let column_by_name_fn = methods::column_by_name::make(methods::column_by_name::Args {
        root,
        mode,
        generics,
        table_data_name,
        columns,
    });
    let into_column_by_name_fn =
        methods::into_column_by_name::make(methods::into_column_by_name::Args {
            root,
            mode,
            generics,
            table_data_name,
            columns,
        });
    let rows_fn = methods::rows::make(methods::rows::Args { root, columns });
    let row_fn = methods::row::make(methods::row::Args { root, columns });
    let insert_row_fn = methods::insert_row::make(methods::insert_row::Args {
        root,
        generics,
        columns,
        origin_struct_name,
        table_data_name,
    });
    let push_row_fn = methods::push_row::make(methods::push_row::Args {
        root,
        generics,
        origin_struct_name,
    });
    let remove_row_fn = methods::remove_row::make(methods::remove_row::Args {
        root,
        generics,
        columns,
        origin_struct_name,
        style,
    });
    let pop_row_fn = methods::pop_row::make(methods::pop_row::Args {
        root,
        generics,
        origin_struct_name,
    });

    let cell_fns = methods::make_cell_fns(root, style, table_data_name, columns);
    let mut_cell_fns = methods::make_mut_cell_fns(root, style, table_data_name, columns);
    let replace_cell_fns = methods::make_replace_cell_fns(root, style, table_data_name, columns);
    let column_fns = methods::make_column_fns(root, style, table_data_name, columns);
    let into_column_fns = methods::make_into_column_fns(root, style, table_data_name, columns);

    parse_quote! {
        #[automatically_derived]
        impl #impl_generics #table_name #ty_generics #where_clause {
            #column_names_const

            #new_fn
            #column_by_name_fn
            #into_column_by_name_fn
            #rows_fn
            #row_fn
            #insert_row_fn
            #push_row_fn
            #remove_row_fn
            #pop_row_fn

            #(
                #cell_fns
                #mut_cell_fns
                #replace_cell_fns
                #column_fns
                #into_column_fns
            )*
        }
    }
}
