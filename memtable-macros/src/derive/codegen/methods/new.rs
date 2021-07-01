use syn::{parse_quote, ItemFn};

pub struct Args {}

pub fn make(_args: Args) -> ItemFn {
    parse_quote! {
        pub fn new() -> Self {
            <Self as ::core::default::Default>::default()
        }
    }
}
