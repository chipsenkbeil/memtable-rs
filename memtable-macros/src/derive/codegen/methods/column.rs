use super::{utils, TableMode};
use syn::{parse_quote, ExprClosure, Ident, ItemFn, Path, Type};

pub struct Args<'a> {
    pub root: &'a Path,
    pub mode: TableMode,
    pub idx: &'a syn::Index,
    pub method_name: &'a Ident,
    pub variant_ty: &'a Type,
    pub table_data_name: &'a Ident,
    pub as_variant: &'a Ident,
    pub into_variant: &'a Ident,
}

pub fn make(args: Args) -> ItemFn {
    let Args {
        root,
        mode,
        idx,
        method_name,
        variant_ty,
        table_data_name,
        as_variant,
        into_variant,
    } = args;

    let iter_item_ty: Type = match mode {
        TableMode::Ref => parse_quote!(&#variant_ty),
        TableMode::Owned => parse_quote!(#variant_ty),
        TableMode::Mixed => parse_quote!(#root::RefOrOwned<'_, #variant_ty>),
    };

    let msg_1 = utils::using_ref_got_owned_str();
    let msg_2 = utils::using_owned_got_ref_str();
    let map_closure: ExprClosure = match mode {
        TableMode::Ref => parse_quote! {
            |x| #table_data_name::#as_variant(
                x.into_borrowed().expect(#msg_1)
            )
        },
        TableMode::Owned => parse_quote! {
            |x| #table_data_name::#into_variant(
                x.into_owned().expect(#msg_2)
            )
        },
        TableMode::Mixed => parse_quote! {
            |x| match x {
                #root::RefOrOwned::Borrowed(x) => #table_data_name::#as_variant(x).map(
                    #root::RefOrOwned::Borrowed,
                ),
                #root::RefOrOwned::Owned(x) => #table_data_name::#into_variant(x).map(
                    #root::RefOrOwned::Owned,
                ),
            }
        },
    };

    parse_quote! {
        pub fn #method_name(&self) -> impl ::std::iter::Iterator<Item = #iter_item_ty> {
            let iter = #root::Table::column(&self.0, #idx);
            ::std::iter::Iterator::filter_map(iter, #map_closure)
        }
    }
}