use super::{utils, TableColumn};
use syn::{parse_quote, Generics, Ident, ItemFn, Path};

pub struct Args<'a> {
    pub root: &'a Path,
    pub generics: &'a Generics,
    pub columns: &'a [&'a TableColumn],
    pub origin_struct_name: &'a Ident,
}

pub fn make(args: Args) -> ItemFn {
    let Args {
        root,
        generics,
        columns,
        origin_struct_name,
    } = args;

    let (_, ty_generics, _) = generics.split_for_impl();
    let fields = utils::make_field_tokens(columns);
    let utils::VariantMethodIdents { into_variant, .. } =
        utils::make_variant_method_idents(columns);
    let bug_msg = utils::bug_str();

    parse_quote! {
        /// Removes the row at the specified position, shifting up all rows after it
        pub fn remove_row(
            &mut self,
            row: ::std::primitive::usize,
        ) -> ::std::option::Option<#origin_struct_name #ty_generics> {
            if row < #root::Table::row_cnt(&self.0) {
                let mut row = #root::Table::remove_row(&mut self.0, row);

                // NOTE: Because we don't allow access to the underlying table
                //       at the level where the cell enum can be changed to
                //       another type, this should NEVER fail. We want to rely
                //       on that guarantee as it would be considered corrupt
                //       if the data is removed (by single cell) or changes
                //       types underneath.
                ::std::option::Option::Some(#origin_struct_name {
                    #(
                        #fields: ::std::iter::Iterator::next(&mut row)
                            .expect(#bug_msg).#into_variant().expect(#bug_msg)
                    ),*
                })
            } else {
                ::std::option::Option::None
            }
        }
    }
}
