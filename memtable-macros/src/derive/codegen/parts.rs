use super::{utils, TableColumn};
use syn::{parse_quote, Generics, Ident, ItemImpl};

pub struct Args<'a> {
    pub origin_struct_name: &'a Ident,
    pub generics: &'a Generics,
    pub columns: &'a [&'a TableColumn],
}

pub fn make(args: Args) -> (ItemImpl, ItemImpl) {
    let Args {
        origin_struct_name,
        generics,
        columns,
    } = args;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let field = utils::make_field_tokens(columns);
    let ty = utils::make_variant_types(columns);

    let struct_to_parts: ItemImpl = parse_quote! {
        #[automatically_derived]
        impl #impl_generics ::std::convert::From<#origin_struct_name #ty_generics>
            for (#(#ty),*) #where_clause
        {
            /// Convert from struct to tuple of fields
            fn from(x: #origin_struct_name #ty_generics) -> (#(#ty),*) {
                (#(x.#field),*)
            }
        }
    };

    let parts_to_struct: ItemImpl = parse_quote! {
        #[automatically_derived]
        impl #impl_generics ::std::convert::From<(#(#ty),*)>
            for #origin_struct_name #ty_generics #where_clause
        {
            /// Convert from tuple of fields to struct
            fn from((#(#field),*): (#(#ty),*)) -> Self {
                Self { #(#field),* }
            }
        }
    };

    (struct_to_parts, parts_to_struct)
}
