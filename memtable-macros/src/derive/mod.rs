use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_quote, Data, DeriveInput, Generics, Ident, ItemEnum, ItemImpl, Path, Visibility};
use voca_rs::case;

mod data;
mod utils;

use data::{StructTable, TableColumn};

pub fn do_derive_table(root: Path, input: DeriveInput) -> darling::Result<TokenStream> {
    match &input.data {
        Data::Struct(_) => Ok(derive_table_from_struct(
            root,
            StructTable::from_derive_input(&input)?,
        )),
        Data::Enum(_) => Err(darling::Error::custom("Enums are not supported").with_span(&input)),
        Data::Union(_) => Err(darling::Error::custom("Unions are not supported").with_span(&input)),
    }
}

fn derive_table_from_struct(root: Path, table: StructTable) -> TokenStream {
    let vis = &table.vis;
    let (_, ty_generics, where_clause) = table.generics.split_for_impl();

    let table_name = table
        .name
        .as_ref()
        .map(|x| format_ident!("{}", x))
        .unwrap_or_else(|| format_ident!("{}Table", &table.ident));
    let table_data_name = table
        .data_name
        .as_ref()
        .map(|x| format_ident!("{}", x))
        .unwrap_or_else(|| format_ident!("{}TableData", &table.ident));
    let columns = &table.data.as_ref().take_struct().unwrap().fields;
    let (item_enum, item_enum_impl) =
        make_table_data_enum(vis, &table_data_name, &table.generics, columns);

    let common_traits = make_common_traits(
        &root,
        &table_name,
        &table.generics,
        &table_data_name,
        columns,
    );
    let (struct_to_parts, parts_to_struct) = if table.skip_parts.is_none() {
        let (x, y) = make_convert_parts(&table.ident, &table.generics, columns);
        (Some(x), Some(y))
    } else {
        (None, None)
    };
    let table_impl = make_table_impl(
        &root,
        &table.ident,
        &table_name,
        &table.generics,
        &table_data_name,
        columns,
    );

    quote! {
        #[automatically_derived]
        #vis struct #table_name #ty_generics(
            #root::Table<self::#table_data_name #ty_generics>
        ) #where_clause;

        #item_enum
        #item_enum_impl
        #common_traits
        #struct_to_parts
        #parts_to_struct
        #table_impl
    }
}

fn make_table_impl(
    root: &Path,
    name: &Ident,
    table_name: &Ident,
    generics: &Generics,
    table_data_name: &Ident,
    columns: &[&TableColumn],
) -> ItemImpl {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let bug_msg = utils::bug_str();

    let variant = utils::make_variant_idents(columns);
    let variant_ty = utils::make_variant_types(columns);
    let snake_case_variant = utils::make_snake_idents(columns);
    let fields = utils::make_field_tokens(columns);
    let idx: Vec<usize> = (0..variant.len()).into_iter().collect();
    let utils::VariantMethodIdents {
        as_variant,
        as_mut_variant,
        into_variant,
        ..
    } = utils::make_variant_method_idents(columns);

    let get_cell: Vec<Ident> = snake_case_variant
        .iter()
        .map(|name| format_ident!("get_{}", name))
        .collect();
    let get_mut_cell: Vec<Ident> = snake_case_variant
        .iter()
        .map(|name| format_ident!("get_mut_{}", name))
        .collect();
    let remove_cell: Vec<Ident> = snake_case_variant
        .iter()
        .map(|name| format_ident!("remove_{}", name))
        .collect();
    let column: Vec<Ident> = snake_case_variant
        .iter()
        .map(|name| format_ident!("{}_column", name))
        .collect();
    let into_column: Vec<Ident> = snake_case_variant
        .iter()
        .map(|name| format_ident!("into_{}_column", name))
        .collect();

    parse_quote! {
        #[automatically_derived]
        impl #impl_generics #table_name #ty_generics #where_clause {
            pub fn new() -> Self {
                <Self as ::std::default::Default>::default()
            }

            /// Returns the numbers of the columns associated with this type of table
            pub fn column_names(&self) -> &[&'static ::std::primitive::str] {
                &[#(::std::stringify!(#variant)),*]
            }

            /// Retrieves a column by its name
            pub fn column_by_name(
                &self,
                name: &::std::primitive::str,
            ) -> ::std::option::Option<#root::Column<self::#table_data_name #ty_generics>> {
                match name {
                    #(
                        ::std::stringify!(#variant) =>
                            ::std::option::Option::Some(self.0.column(#idx)),
                    )*
                    _ => ::std::option::Option::None,
                }
            }

            /// Converts into a column by its name
            pub fn into_column_by_name(
                self,
                name: &::std::primitive::str,
            ) -> ::std::option::Option<#root::IntoColumn<self::#table_data_name #ty_generics>> {
                match name {
                    #(
                        ::std::stringify!(#variant) =>
                            ::std::option::Option::Some(self.0.into_column(#idx)),
                    )*
                    _ => ::std::option::Option::None,
                }
            }

            /// Iterates through each row of the table, returning a tuple of references
            /// to the individual fields
            pub fn typed_rows(&self) -> impl ::std::iter::Iterator<Item = (#(&#variant_ty),*)> {
                ::std::iter::Iterator::map(
                    0..self.0.row_cnt(),
                    move |idx| self.typed_row(idx),
                )
            }

            /// Returns a tuple containing refs to row's data
            pub fn typed_row(&self, row: ::std::primitive::usize) -> (#(&#variant_ty),*) {
                // NOTE: Because we don't allow access to the underlying table
                //       at the level where the cell enum can be changed to
                //       another type, this should NEVER fail. We want to rely
                //       on that guarantee as it would be considered corrupt
                //       if the data changed types underneath.
                (#(
                    self.#get_cell(row).expect(#bug_msg)
                ),*)
            }

            /// Inserts a new row into the table at the given position, shifting down
            /// all rows after it
            pub fn insert_row<RowData: ::std::convert::Into<#name #ty_generics>>(
                &mut self,
                row: ::std::primitive::usize,
                data: RowData,
            ) {
                let data = data.into();
                self.0.insert_row(row, ::std::vec![
                    #(self::#table_data_name::#variant(data.#fields)),*
                ]);
            }

            /// Pushes a row to the end of the table
            pub fn push_row<RowData: ::std::convert::Into<#name #ty_generics>>(
                &mut self,
                data: RowData,
            ) {
                self.insert_row(self.0.row_cnt(), data)
            }

            /// Removes the row at the specified position, shifting up all rows after it
            pub fn remove_row(
                &mut self,
                row: ::std::primitive::usize,
            ) -> ::std::option::Option<#name #ty_generics> {
                if row < self.0.row_cnt() {
                    let mut row = self.0.remove_row(row);

                    // NOTE: Because we don't allow access to the underlying table
                    //       at the level where the cell enum can be changed to
                    //       another type, this should NEVER fail. We want to rely
                    //       on that guarantee as it would be considered corrupt
                    //       if the data is removed (by single cell) or changes
                    //       types underneath.
                    ::std::option::Option::Some(#name {
                        #(
                            #fields: ::std::iter::Iterator::next(
                                &mut row,
                            ).expect(#bug_msg).#into_variant().expect(#bug_msg)
                        ),*
                    })
                } else {
                    ::std::option::Option::None
                }
            }

            /// Pops a row off the end of the table
            pub fn pop_row(&mut self) -> ::std::option::Option<#name #ty_generics> {
                let max_rows = self.0.row_cnt();
                self.remove_row(if max_rows > 0 { max_rows - 1 } else { 0 })
            }

            #(
                pub fn #get_cell(
                    &self,
                    row: ::std::primitive::usize,
                ) -> ::std::option::Option<&#variant_ty> {
                    self.0.get_cell(row, #idx).and_then(self::#table_data_name::#as_variant)
                }

                pub fn #get_mut_cell(
                    &mut self,
                    row: ::std::primitive::usize,
                ) -> ::std::option::Option<&mut #variant_ty> {
                    self.0.get_mut_cell(row, #idx).and_then(self::#table_data_name::#as_mut_variant)
                }

                pub fn #remove_cell(
                    &mut self,
                    row: ::std::primitive::usize,
                ) -> ::std::option::Option<#variant_ty> {
                    self.0.remove_cell(row, #idx).and_then(self::#table_data_name::#into_variant)
                }

                pub fn #column(&self) -> impl ::std::iter::Iterator<Item = &#variant_ty> {
                    let iter = self.0.column(#idx);
                    ::std::iter::Iterator::filter_map(
                        iter,
                        self::#table_data_name::#as_variant,
                    )
                }

                pub fn #into_column(self) -> impl ::std::iter::Iterator<Item = #variant_ty> {
                    let iter = self.0.into_column(#idx);
                    ::std::iter::Iterator::filter_map(
                        iter,
                        self::#table_data_name::#into_variant,
                    )
                }
            )*
        }
    }
}

fn make_common_traits(
    root: &Path,
    name: &Ident,
    generics: &Generics,
    data_name: &Ident,
    columns: &[&TableColumn],
) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let variant = utils::make_variant_idents(columns);
    let ty = utils::make_variant_types(columns);
    let is_ty: Vec<Ident> = utils::make_column_names(columns, case::snake_case)
        .into_iter()
        .map(|name| format_ident!("is_{}", name))
        .collect();
    let idx: Vec<usize> = (0..is_ty.len()).into_iter().collect();

    quote! {
        #[automatically_derived]
        impl #impl_generics ::std::convert::AsRef<#root::Table<self::#data_name #ty_generics>>
            for #name #ty_generics #where_clause
        {
            fn as_ref(&self) -> &#root::Table<self::#data_name #ty_generics> {
                &self.0
            }
        }

        #[automatically_derived]
        impl #impl_generics ::std::ops::Deref for #name #ty_generics #where_clause {
            type Target = #root::Table<self::#data_name #ty_generics>;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        #[automatically_derived]
        impl #impl_generics ::std::convert::From<#name #ty_generics>
            for #root::Table<self::#data_name #ty_generics> #where_clause
        {
            fn from(x: #name #ty_generics) -> Self {
                x.0
            }
        }

        #[automatically_derived]
        impl #impl_generics ::std::default::Default for #name #ty_generics #where_clause {
            fn default() -> Self {
                Self(<#root::Table<self::#data_name #ty_generics> as ::std::default::Default>::default())
            }
        }

        #[automatically_derived]
        impl #impl_generics ::std::convert::TryFrom<#root::Table<self::#data_name #ty_generics>>
            for #name #ty_generics #where_clause
        {
            type Error = &'static ::std::primitive::str;

            fn try_from(
                table: #root::Table<self::#data_name #ty_generics>,
            ) -> ::std::result::Result<Self, Self::Error> {
                #(
                    for cell in table.column(#idx) {
                        if !cell.#is_ty() {
                            return ::std::result::Result::Err(
                                ::std::concat!(
                                    "Cell in column ",
                                    ::std::stringify!(#idx),
                                    "/",
                                    ::std::stringify!(#variant),
                                    " is not of type ",
                                    ::std::stringify!(#ty),
                                )
                            );
                        }
                    }
                )*

                ::std::result::Result::Ok(Self(table))
            }
        }
    }
}

fn make_convert_parts(
    name: &Ident,
    generics: &Generics,
    columns: &[&TableColumn],
) -> (ItemImpl, ItemImpl) {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let field = utils::make_field_tokens(columns);
    let ty = utils::make_variant_types(columns);

    let struct_to_parts: ItemImpl = parse_quote! {
        #[automatically_derived]
        impl #impl_generics ::std::convert::From<#name #ty_generics>
            for (#(#ty),*) #where_clause
        {
            /// Convert from struct to tuple of fields
            fn from(x: #name #ty_generics) -> (#(#ty),*) {
                (#(x.#field),*)
            }
        }
    };

    let parts_to_struct: ItemImpl = parse_quote! {
        #[automatically_derived]
        impl #impl_generics ::std::convert::From<(#(#ty),*)>
            for #name #ty_generics #where_clause
        {
            /// Convert from tuple of fields to struct
            fn from((#(#field),*): (#(#ty),*)) -> Self {
                Self { #(#field),* }
            }
        }
    };

    (struct_to_parts, parts_to_struct)
}

fn make_table_data_enum(
    vis: &Visibility,
    name: &Ident,
    generics: &Generics,
    columns: &[&TableColumn],
) -> (ItemEnum, ItemImpl) {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let variant = utils::make_variant_idents(columns);
    let variant_ty = utils::make_variant_types(columns);
    let utils::VariantMethodIdents {
        is_variant,
        as_variant,
        as_mut_variant,
        into_variant,
    } = utils::make_variant_method_idents(columns);

    let item_enum = parse_quote! {
        #[automatically_derived]
        #vis enum #name #ty_generics #where_clause {
            #(#variant(#variant_ty)),*
        }
    };

    let item_impl = parse_quote! {
        #[automatically_derived]
        impl #impl_generics #name #ty_generics #where_clause {
            #(
                pub fn #is_variant(&self) -> ::std::primitive::bool {
                    match self {
                        Self::#variant(_) => true,
                        _ => false,
                    }
                }

                pub fn #as_variant(&self) -> ::std::option::Option<&#variant_ty> {
                    match self {
                        Self::#variant(x) => ::std::option::Option::Some(x),
                        _ => ::std::option::Option::None,
                    }
                }

                pub fn #as_mut_variant(&mut self) -> ::std::option::Option<&mut #variant_ty> {
                    match self {
                        Self::#variant(x) => ::std::option::Option::Some(x),
                        _ => ::std::option::Option::None,
                    }
                }

                pub fn #into_variant(self) -> ::std::option::Option<#variant_ty> {
                    match self {
                        Self::#variant(x) => ::std::option::Option::Some(x),
                        _ => ::std::option::Option::None,
                    }
                }
            )*
        }
    };

    (item_enum, item_impl)
}
