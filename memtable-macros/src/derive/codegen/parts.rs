use super::{utils, TableColumn};
use darling::ast::Style;
use syn::{parse_quote, Expr, Generics, Ident, ItemImpl};

pub struct Args<'a> {
    pub origin_struct_name: &'a Ident,
    pub generics: &'a Generics,
    pub columns: &'a [&'a TableColumn],
    pub style: Style,
}

pub fn make(args: Args) -> (ItemImpl, ItemImpl) {
    let Args {
        origin_struct_name,
        generics,
        columns,
        style,
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

    let (args, body): (, Expr) = match style {
        Style::Tuple => parse_quote!(Self ( #(#field),* )),
        Style::Struct => parse_quote!(Self { #(#field),* }),
        Style::Unit => unreachable!(),
    };

    let parts_to_struct: ItemImpl = parse_quote! {
        #[automatically_derived]
        impl #impl_generics ::std::convert::From<(#(#ty),*)>
            for #origin_struct_name #ty_generics #where_clause
        {
            /// Convert from tuple of fields to struct
            fn from((#(#field),*): (#(#ty),*)) -> Self {
                #body
            }
        }
    };

    (struct_to_parts, parts_to_struct)
}
