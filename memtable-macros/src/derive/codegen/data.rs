use super::{utils, TableColumn, TableMode};
use darling::{ast::Style, util::PathList};
use quote::quote;
use syn::{parse_quote, Generics, Ident, ItemEnum, ItemImpl, Visibility};

pub struct Args<'a> {
    pub mode: TableMode,
    pub style: Style,
    pub vis: &'a Visibility,
    pub table_data_name: &'a Ident,
    pub generics: &'a Generics,
    pub derive: Option<&'a PathList>,
    pub columns: &'a [&'a TableColumn],
}

pub struct Return {
    pub definition: ItemEnum,
    pub core_impl: ItemImpl,
    pub default_impl: Option<ItemImpl>,
}

pub fn make(args: Args) -> Return {
    let Args {
        mode,
        style,
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
    } = utils::make_variant_method_idents(style, columns);

    // Support forwading derive attributes
    let derive_attr = derive
        .filter(|list| !list.is_empty())
        .map(|list| quote!(#[derive(#(#list),*)]));

    let definition = parse_quote! {
        #[automatically_derived]
        #derive_attr
        #vis enum #table_data_name #impl_generics #where_clause {
            #(#variant(#variant_ty)),*
        }
    };

    // All modes other than dynamic require the data to implement default,
    // which we do by hand-crafting an impl (can't derive on enum).
    //
    // TODO: By default, we'll attempt to use the first variant's value as the
    //       default; however, we should support letting the user choose the
    //       variant via an attribute on the column
    let default_impl: Option<ItemImpl> = if !matches!(mode, TableMode::Dynamic) {
        let body = if variant.is_empty() {
            quote!(::core::compile_error!("At least one field is required!"))
        } else {
            let name = &variant[0];
            let ty = &variant_ty[0];
            quote!(Self::#name(<#ty as ::core::default::Default>::default()))
        };

        Some(parse_quote! {
            #[automatically_derived]
            impl #impl_generics ::core::default::Default
                for #table_data_name #ty_generics #where_clause
            {
                fn default() -> Self {
                    #body
                }
            }
        })
    } else {
        None
    };

    let core_impl = parse_quote! {
        #[automatically_derived]
        impl #impl_generics #table_data_name #ty_generics #where_clause {
            #(
                pub fn #is_variant(&self) -> ::core::primitive::bool {
                    match self {
                        Self::#variant(_) => true,
                        _ => false,
                    }
                }

                pub fn #as_variant(&self) -> ::core::option::Option<&#variant_ty> {
                    match self {
                        Self::#variant(x) => ::core::option::Option::Some(x),
                        _ => ::core::option::Option::None,
                    }
                }

                pub fn #as_mut_variant(&mut self) -> ::core::option::Option<&mut #variant_ty> {
                    match self {
                        Self::#variant(x) => ::core::option::Option::Some(x),
                        _ => ::core::option::Option::None,
                    }
                }

                pub fn #into_variant(self) -> ::core::option::Option<#variant_ty> {
                    match self {
                        Self::#variant(x) => ::core::option::Option::Some(x),
                        _ => ::core::option::Option::None,
                    }
                }
            )*
        }
    };

    Return {
        definition,
        core_impl,
        default_impl,
    }
}
