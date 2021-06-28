use super::{utils, TableMode};
use syn::{parse_quote, Generics, Ident, ItemImpl, Path};

pub struct Args<'a> {
    pub root: &'a Path,
    pub mode: TableMode,
    pub table_name: &'a Ident,
    pub generics: &'a Generics,
    pub table_data_name: &'a Ident,
    pub col_cnt: usize,
}

pub fn make(args: Args) -> ItemImpl {
    let Args {
        root,
        mode,
        table_name,
        generics,
        table_data_name,
        col_cnt,
    } = args;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let inner_table_ty =
        utils::make_inner_table_type(root, mode, table_data_name, generics, col_cnt);

    parse_quote! {
        #[automatically_derived]
        impl #impl_generics ::std::default::Default
            for #table_name #ty_generics #where_clause
        {
            fn default() -> Self {
                Self(<
                    #inner_table_ty as
                    ::std::default::Default
                >::default())
            }
        }
    }
}
