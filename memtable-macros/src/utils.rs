use proc_macro2::{Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use syn::{parse_macro_input, parse_quote, DeriveInput, Ident, Path};

/// Produces a token stream in the form of `::memtable` or renamed version
pub fn entity_crate() -> darling::Result<Path> {
    get_crate("memtable", true)
}

/// If `no_error` is true, instead of returning an error when the crate is
/// missing the function will return a path to what the crate would like to
/// be; only really useful in tests
fn get_crate(cname: &str, no_error: bool) -> darling::Result<Path> {
    let res = crate_name(cname)
        .map(|found_crate| match found_crate {
            FoundCrate::Itself => {
                let crate_ident = Ident::new(&cname, Span::mixed_site());
                parse_quote!(::#crate_ident)
            }
            FoundCrate::Name(name) => {
                let crate_ident = Ident::new(&name, Span::mixed_site());
                parse_quote!(::#crate_ident)
            }
        })
        .map_err(|msg| darling::Error::custom(msg).with_span(&Span::mixed_site()));

    match res {
        Err(_) if no_error => {
            let crate_ident = Ident::new(&cname, Span::mixed_site());
            Ok(parse_quote!(::#crate_ident))
        }
        x => x,
    }
}

/// Main helper called within each derive macro
pub fn do_derive(
    f: fn(Path, DeriveInput) -> darling::Result<TokenStream>,
) -> impl Fn(proc_macro::TokenStream) -> proc_macro::TokenStream {
    move |input: proc_macro::TokenStream| {
        let input = parse_macro_input!(input as DeriveInput);

        let expanded = entity_crate()
            .and_then(|root| f(root, input))
            .unwrap_or_else(|x| x.write_errors());

        proc_macro::TokenStream::from(expanded)
    }
}
