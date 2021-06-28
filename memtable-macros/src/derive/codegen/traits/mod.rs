pub mod as_ref;
pub mod default;
pub mod deref;
pub mod from;
pub mod table;
pub mod try_from;

use super::{utils, TableColumn};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Generics, Ident, Path};

pub struct CommonArgs<'a> {
    pub root: &'a Path,
    pub table_name: &'a Ident,
    pub generics: &'a Generics,
    pub table_data_name: &'a Ident,
    pub columns: &'a [&'a TableColumn],
}

pub fn make_common(args: CommonArgs) -> TokenStream {
    let CommonArgs {
        root,
        table_name,
        generics,
        table_data_name,
        columns,
    } = args;

    let as_ref_trait = as_ref::make(as_ref::Args {
        root,
        table_name,
        generics,
        table_data_name,
    });

    let default_trait = default::make(default::Args {
        root,
        table_name,
        generics,
        table_data_name,
    });

    let deref_trait = deref::make(deref::Args {
        root,
        table_name,
        generics,
        table_data_name,
    });

    let from_trait = from::make(from::Args {
        root,
        table_name,
        generics,
        table_data_name,
    });

    let try_from_trait = try_from::make(try_from::Args {
        root,
        table_name,
        generics,
        table_data_name,
        columns,
    });

    quote! {
        #as_ref_trait
        #default_trait
        #deref_trait
        #from_trait
        #try_from_trait
    }
}
