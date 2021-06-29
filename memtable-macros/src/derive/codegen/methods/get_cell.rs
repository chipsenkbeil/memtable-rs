use syn::{parse_quote, Ident, ItemFn, Path, Type};

pub struct Args<'a> {
    pub root: &'a Path,
    pub idx: &'a syn::Index,
    pub method_name: &'a Ident,
    pub variant_ty: &'a Type,
    pub table_data_name: &'a Ident,
    pub as_variant: &'a Ident,
}

pub fn make(args: Args) -> ItemFn {
    let Args {
        root,
        idx,
        method_name,
        variant_ty,
        table_data_name,
        as_variant,
    } = args;

    parse_quote! {
        pub fn #method_name(
            &self,
            row: ::std::primitive::usize,
        ) -> ::std::option::Option<&#variant_ty> {
            #root::Table::get_cell(&self.0, row, #idx)
                .and_then(#table_data_name::#as_variant)
        }
    }
}
