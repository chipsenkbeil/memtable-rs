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
        #[automatically_derived]
        impl #impl_generics ::std::convert::AsRef<#root::MemDynamicTable<#table_data_name #ty_generics>>
            for #table_name #ty_generics #where_clause
        {
            fn as_ref(&self) -> &#root::MemDynamicTable<#table_data_name #ty_generics> {
                &self.0
            }
        }
    }
}
