use syn::{parse_quote, Ident, ItemFn, Path, Type};

pub struct Args<'a> {
    pub root: &'a Path,
    pub idx: &'a syn::Index,
    pub method_name: &'a Ident,
    pub variant_ty: &'a Type,
    pub table_data_name: &'a Ident,
    pub into_variant: &'a Ident,
}

pub fn make(args: Args) -> ItemFn {
    let Args {
        root,
        idx,
        method_name,
        variant_ty,
        table_data_name,
        into_variant,
    } = args;

    parse_quote! {
        pub fn #method_name(self) -> impl ::std::iter::Iterator<Item = #variant_ty> {
            let iter = #root::Table::into_column(self.0, #idx);
            ::std::iter::Iterator::filter_map(
                iter,
                #table_data_name::#into_variant,
            )
        }
    }
}
