use super::{utils, TableColumn};
use darling::util::PathList;
use quote::quote;
use syn::{parse_quote, Generics, Ident, ItemEnum, ItemImpl, Visibility};

pub struct Args<'a> {
    pub vis: &'a Visibility,
    pub table_data_name: &'a Ident,
    pub generics: &'a Generics,
    pub derive: Option<&'a PathList>,
    pub columns: &'a [&'a TableColumn],
}

pub fn make(args: Args) -> (ItemEnum, ItemImpl) {
    let Args {
        vis,
        table_data_name,
        generics,
        derive,
        columns,
    } = args;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let variant = utils::make_variant_idents(columns);
    let variant_ty = utils::make_variant_types(columns);
    let utils::VariantMethodIdents {
        is_variant,
        as_variant,
        as_mut_variant,
        into_variant,
    } = utils::make_variant_method_idents(columns);

    let derive_attr = derive
        .filter(|list| !list.is_empty())
        .map(|list| quote!(#[derive(#(#list),*)]));

    let item_enum = parse_quote! {
        #[automatically_derived]
        #derive_attr
        #vis enum #table_data_name #ty_generics #where_clause {
            #(#variant(#variant_ty)),*
        }
    };

    let item_impl = parse_quote! {
        #[automatically_derived]
        impl #impl_generics #table_data_name #ty_generics #where_clause {
            #(
                pub fn #is_variant(&self) -> ::std::primitive::bool {
                    match self {
                        Self::#variant(_) => true,
                        _ => false,
                    }
                }

                pub fn #as_variant(&self) -> ::std::option::Option<&#variant_ty> {
                    match self {
                        Self::#variant(x) => ::std::option::Option::Some(x),
                        _ => ::std::option::Option::None,
                    }
                }

                pub fn #as_mut_variant(&mut self) -> ::std::option::Option<&mut #variant_ty> {
                    match self {
                        Self::#variant(x) => ::std::option::Option::Some(x),
                        _ => ::std::option::Option::None,
                    }
                }

                pub fn #into_variant(self) -> ::std::option::Option<#variant_ty> {
                    match self {
                        Self::#variant(x) => ::std::option::Option::Some(x),
                        _ => ::std::option::Option::None,
                    }
                }
            )*
        }
    };

    (item_enum, item_impl)
}
