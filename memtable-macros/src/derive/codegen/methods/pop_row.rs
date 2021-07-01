use syn::{parse_quote, Generics, Ident, ItemFn, Path};

pub struct Args<'a> {
    pub root: &'a Path,
    pub generics: &'a Generics,
    pub origin_struct_name: &'a Ident,
}

pub fn make(args: Args) -> ItemFn {
    let Args {
        root,
        generics,
        origin_struct_name,
    } = args;

    let (_, ty_generics, _) = generics.split_for_impl();

    parse_quote! {
        /// Pops a row off the end of the table
        pub fn pop_row(&mut self) -> ::core::option::Option<#origin_struct_name #ty_generics> {
            let max_rows = #root::Table::row_cnt(&self.0);
            self.remove_row(if max_rows > 0 { max_rows - 1 } else { 0 })
        }
    }
}
