use super::{utils, TableColumn};
use darling::ast::Style;
use syn::{parse_quote, Expr, Generics, Ident, ItemFn, Path};

pub struct Args<'a> {
    pub root: &'a Path,
    pub generics: &'a Generics,
    pub columns: &'a [&'a TableColumn],
    pub origin_struct_name: &'a Ident,
    pub style: Style,
}

pub fn make(args: Args) -> ItemFn {
    let Args {
        root,
        generics,
        columns,
        origin_struct_name,
        style,
    } = args;

    let (_, ty_generics, _) = generics.split_for_impl();
    let fields = utils::make_field_tokens(columns);
    let utils::VariantMethodIdents { into_variant, .. } =
        utils::make_variant_method_idents(style, columns);
    let bug_msg = utils::bug_str();

    let create_struct_expr: Expr = match style {
        Style::Tuple => parse_quote! {
            #origin_struct_name(#(
                ::core::iter::Iterator::next(&mut iter)
                    .expect(#bug_msg)
                    .#into_variant()
                    .expect(#bug_msg)
            ),*)
        },
        Style::Struct => parse_quote! {
            #origin_struct_name {#(
                #fields: ::core::iter::Iterator::next(&mut iter)
                    .expect(#bug_msg)
                    .#into_variant()
                    .expect(#bug_msg)
            ),*}
        },
        Style::Unit => unreachable!(),
    };

    parse_quote! {
        /// Removes the row at the specified position, shifting up all rows after it
        pub fn remove_row(
            &mut self,
            row: ::core::primitive::usize,
        ) -> ::core::option::Option<#origin_struct_name #ty_generics> {
            #root::Table::remove_row(&mut self.0, row).and_then(|row| {
                // Build an iterator so we can consume the row values
                let mut iter = ::core::iter::IntoIterator::into_iter(row);

                // NOTE: Because we don't allow access to the underlying table
                //       at the level where the cell enum can be changed to
                //       another type, this should NEVER fail. We want to rely
                //       on that guarantee as it would be considered corrupt
                //       if the data is removed (by single cell) or changes
                //       types underneath.
                ::core::option::Option::Some(#create_struct_expr)
            })
        }
    }
}
