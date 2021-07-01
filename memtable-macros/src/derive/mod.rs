use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Path};

mod codegen;
mod parse;

use parse::{StructTable, TableColumn, TableMode};

pub fn do_derive_table(root: Path, input: DeriveInput) -> darling::Result<TokenStream> {
    match &input.data {
        Data::Struct(_) => Ok(derive_table_from_struct(
            root,
            StructTable::from_derive_input(&input)?,
        )),
        Data::Enum(_) => Err(darling::Error::custom("Enums are not supported").with_span(&input)),
        Data::Union(_) => Err(darling::Error::custom("Unions are not supported").with_span(&input)),
    }
}

fn derive_table_from_struct(root: Path, table: StructTable) -> TokenStream {
    let vis = &table.vis;
    let (impl_generics, _, where_clause) = table.generics.split_for_impl();

    let table_name = table.to_table_name();
    let table_data_name = table.to_table_data_name();
    let generics = &table.generics;
    let columns = table.columns();
    let mode = table.mode;
    let style = table.as_style();

    let codegen::data::Return {
        definition: data_definition,
        core_impl: data_core_impl,
        default_impl: data_default_impl,
    } = codegen::data::make(codegen::data::Args {
        vis,
        style,
        mode,
        table_data_name: &table_data_name,
        generics,
        derive: table.data_attr.as_ref().and_then(|x| x.derive.as_ref()),
        columns: &columns,
    });

    let common_traits = codegen::traits::make_common(codegen::traits::CommonArgs {
        root: &root,
        mode,
        table_name: &table_name,
        generics,
        table_data_name: &table_data_name,
        columns: &columns,
    });

    let table_trait = codegen::traits::table::make(codegen::traits::table::Args {
        root: &root,
        mode,
        table_name: &table_name,
        generics,
        table_data_name: &table_data_name,
        columns: &columns,
    });

    let (struct_to_parts, parts_to_struct) = if table.skip_parts.is_none() {
        let (x, y) = codegen::parts::make(codegen::parts::Args {
            origin_struct_name: &table.ident,
            generics,
            columns: &columns,
            style,
        });
        (Some(x), Some(y))
    } else {
        (None, None)
    };

    let table_impl = codegen::make_table_impl(codegen::TableImplArgs {
        root: &root,
        mode,
        style,
        origin_struct_name: &table.ident,
        table_name: &table_name,
        generics: &table.generics,
        table_data_name: &table_data_name,
        columns: &columns,
    });

    let inner_table_ty = codegen::utils::make_inner_table_type(
        &root,
        mode,
        &table_data_name,
        &table.generics,
        columns.len(),
    );

    let derive_attr = table
        .derive
        .filter(|list| !list.is_empty())
        .map(|list| quote!(#[derive(#(#list),*)]));

    quote! {
        #[automatically_derived]
        #derive_attr
        #vis struct #table_name #impl_generics(#inner_table_ty) #where_clause;

        #data_definition
        #data_core_impl
        #data_default_impl
        #common_traits
        #struct_to_parts
        #parts_to_struct
        #table_trait
        #table_impl
    }
}
