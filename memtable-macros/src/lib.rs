#![forbid(unsafe_code)]

mod derive;
mod utils;

#[proc_macro_derive(Table, attributes(tbl))]
pub fn derive_table(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    utils::do_derive(derive::do_derive_table)(input)
}
