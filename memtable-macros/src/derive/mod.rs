use darling::{util::PathList, FromDeriveInput};
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
        .data_attr
        .as_ref()
        .and_then(|x| x.name.as_ref())
        .map(|x| format_ident!("{}", x))
        .unwrap_or_else(|| format_ident!("{}TableData", &table.ident));
    let columns = &table.data.as_ref().take_struct().unwrap().fields;
    let (item_enum, item_enum_impl) = make_table_data_enum(
        vis,
        &table_data_name,
        &table.generics,
        table.data_attr.as_ref().and_then(|x| x.derive.as_ref()),
        columns,
    );

    let common_traits = make_common_traits(
        &root,
        &table_name,
        &table.generics,
        &table_data_name,
        columns,
    );
    let table_trait = make_table_trait(&root, &table_name, &table.generics, &table_data_name);
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

    let derive_attr = table
        .derive
        .filter(|list| !list.is_empty())
        .map(|list| quote!(#[derive(#(#list),*)]));

    quote! {
        #[automatically_derived]
        #derive_attr
        #vis struct #table_name #ty_generics(
            #root::MemTable<#table_data_name #ty_generics>
        ) #where_clause;

        #item_enum
        #item_enum_impl
        #common_traits
        #struct_to_parts
        #parts_to_struct
        #table_trait
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

    let column_name = utils::make_column_names(columns, ToString::to_string);
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
    let replace_cell: Vec<Ident> = snake_case_variant
        .iter()
        .map(|name| format_ident!("replace_{}", name))
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
            pub const fn column_names() -> &'static [&'static ::std::primitive::str] {
                &[#(#column_name),*]
            }

            /// Retrieves a column by its name
            pub fn column_by_name(
                &self,
                name: &::std::primitive::str,
            ) -> ::std::option::Option<#root::iter::Column<
                #table_data_name #ty_generics,
                #root::MemTable<#table_data_name #ty_generics>,
            >> {
                match name {
                    #(
                        #column_name => ::std::option::Option::Some(
                            #root::Table::column(&self.0, #idx)
                        ),
                    )*
                    _ => ::std::option::Option::None,
                }
            }

            /// Converts into a column by its name
            pub fn into_column_by_name(
                self,
                name: &::std::primitive::str,
            ) -> ::std::option::Option<#root::iter::IntoColumn<
                #table_data_name #ty_generics,
                #root::MemTable<#table_data_name #ty_generics>,
            >> {
                match name {
                    #(
                        #column_name => ::std::option::Option::Some(
                            #root::Table::into_column(self.0, #idx)
                        ),
                    )*
                    _ => ::std::option::Option::None,
                }
            }

            /// Iterates through each row of the table, returning a tuple of references
            /// to the individual fields
            pub fn rows(&self) -> impl ::std::iter::Iterator<Item = (#(&#variant_ty),*)> {
                // NOTE: The expect(...) should never happen as we should have
                //       all of the rows available in the described range
                ::std::iter::Iterator::map(
                    0..#root::Table::row_cnt(&self.0),
                    move |idx| self.row(idx).expect(#bug_msg),
                )
            }

            /// Returns a tuple containing refs to row's data
            pub fn row(&self, row: ::std::primitive::usize) -> ::std::option::Option<(#(&#variant_ty),*)> {
                // NOTE: Because we don't allow access to the underlying table
                //       at the level where the cell enum can be changed to
                //       another type, this should NEVER fail. We want to rely
                //       on that guarantee as it would be considered corrupt
                //       if the data changed types underneath.
                if row < #root::Table::row_cnt(&self.0) {
                    ::std::option::Option::Some(
                        (#(
                            self.#get_cell(row).expect(#bug_msg)
                        ),*)
                    )
                } else {
                    ::std::option::Option::None
                }
            }

            /// Inserts a new row into the table at the given position, shifting down
            /// all rows after it
            pub fn insert_row<__RowData: ::std::convert::Into<#name #ty_generics>>(
                &mut self,
                row: ::std::primitive::usize,
                data: __RowData,
            ) {
                let data = data.into();
                #root::Table::insert_row(&mut self.0, row, ::std::vec![
                    #(#table_data_name::#variant(data.#fields)),*
                ]);
            }

            /// Pushes a row to the end of the table
            pub fn push_row<__RowData: ::std::convert::Into<#name #ty_generics>>(
                &mut self,
                data: __RowData,
            ) {
                self.insert_row(#root::Table::row_cnt(&self.0), data)
            }

            /// Removes the row at the specified position, shifting up all rows after it
            pub fn remove_row(
                &mut self,
                row: ::std::primitive::usize,
            ) -> ::std::option::Option<#name #ty_generics> {
                if row < #root::Table::row_cnt(&self.0) {
                    let mut row = #root::Table::remove_row(&mut self.0, row);

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
                let max_rows = #root::Table::row_cnt(&self.0);
                self.remove_row(if max_rows > 0 { max_rows - 1 } else { 0 })
            }

            #(
                pub fn #get_cell(
                    &self,
                    row: ::std::primitive::usize,
                ) -> ::std::option::Option<&#variant_ty> {
                    #root::Table::get_cell(&self.0, row, #idx)
                        .and_then(#table_data_name::#as_variant)
                }

                pub fn #get_mut_cell(
                    &mut self,
                    row: ::std::primitive::usize,
                ) -> ::std::option::Option<&mut #variant_ty> {
                    #root::Table::get_mut_cell(&mut self.0, row, #idx)
                        .and_then(#table_data_name::#as_mut_variant)
                }

                /// Swaps the current cell value with the provided one, doing nothing
                /// if there is no cell at the specified row for the explicit column
                pub fn #replace_cell<__Value: ::std::convert::Into<#variant_ty>>(
                    &mut self,
                    row: ::std::primitive::usize,
                    value: __Value,
                ) -> ::std::option::Option<#variant_ty> {
                    if row < #root::Table::row_cnt(&self.0) {
                        #root::Table::insert_cell(
                            &mut self.0,
                            row,
                            #idx,
                            #table_data_name::#variant(value.into()),
                        ).and_then(#table_data_name::#into_variant)
                    } else {
                        ::std::option::Option::None
                    }
                }

                pub fn #column(&self) -> impl ::std::iter::Iterator<Item = &#variant_ty> {
                    let iter = #root::Table::column(&self.0, #idx);
                    ::std::iter::Iterator::filter_map(
                        iter,
                        #table_data_name::#as_variant,
                    )
                }

                pub fn #into_column(self) -> impl ::std::iter::Iterator<Item = #variant_ty> {
                    let iter = #root::Table::into_column(self.0, #idx);
                    ::std::iter::Iterator::filter_map(
                        iter,
                        #table_data_name::#into_variant,
                    )
                }
            )*
        }
    }
}

fn make_table_trait(root: &Path, name: &Ident, generics: &Generics, data_name: &Ident) -> ItemImpl {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    parse_quote! {
        impl #impl_generics #root::Table for #name #ty_generics #where_clause {
            type Data = #data_name #ty_generics;

            fn row_cnt(&self) -> ::std::primitive::usize {
                #root::Table::row_cnt(&self.0)
            }

            fn col_cnt(&self) -> ::std::primitive::usize {
                #root::Table::col_cnt(&self.0)
            }

            fn get_cell(
                &self,
                row: ::std::primitive::usize,
                col: ::std::primitive::usize,
            ) -> ::std::option::Option<&Self::Data> {
                #root::Table::get_cell(&self.0, row, col)
            }

            fn get_mut_cell(
                &mut self,
                row: ::std::primitive::usize,
                col: ::std::primitive::usize,
            ) -> ::std::option::Option<&mut Self::Data> {
                #root::Table::get_mut_cell(&mut self.0, row, col)
            }

            fn insert_cell(
                &mut self,
                row: ::std::primitive::usize,
                col: ::std::primitive::usize,
                value: Self::Data,
            ) -> ::std::option::Option<Self::Data> {
                #root::Table::insert_cell(&mut self.0, row, col, value)
            }

            fn remove_cell(
                &mut self,
                row: ::std::primitive::usize,
                col: ::std::primitive::usize,
            ) -> ::std::option::Option<Self::Data> {
                #root::Table::remove_cell(&mut self.0, row, col)
            }

            fn set_row_capacity(&mut self, capacity: ::std::primitive::usize) {
                #root::Table::set_row_capacity(&mut self.0, capacity);
            }

            fn set_column_capacity(&mut self, capacity: ::std::primitive::usize) {
                #root::Table::set_column_capacity(&mut self.0, capacity);
            }
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
        impl #impl_generics ::std::convert::AsRef<#root::MemTable<#data_name #ty_generics>>
            for #name #ty_generics #where_clause
        {
            fn as_ref(&self) -> &#root::MemTable<#data_name #ty_generics> {
                &self.0
            }
        }

        #[automatically_derived]
        impl #impl_generics ::std::ops::Deref for #name #ty_generics #where_clause {
            type Target = #root::MemTable<#data_name #ty_generics>;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        #[automatically_derived]
        impl #impl_generics ::std::convert::From<#name #ty_generics>
            for #root::MemTable<#data_name #ty_generics> #where_clause
        {
            fn from(x: #name #ty_generics) -> Self {
                x.0
            }
        }

        #[automatically_derived]
        impl #impl_generics ::std::default::Default for #name #ty_generics #where_clause {
            fn default() -> Self {
                Self(<#root::MemTable<#data_name #ty_generics> as ::std::default::Default>::default())
            }
        }

        #[automatically_derived]
        impl #impl_generics ::std::convert::TryFrom<#root::MemTable<#data_name #ty_generics>>
            for #name #ty_generics #where_clause
        {
            type Error = &'static ::std::primitive::str;

            fn try_from(
                table: #root::MemTable<#data_name #ty_generics>,
            ) -> ::std::result::Result<Self, Self::Error> {
                for row in 0..#root::Table::row_cnt(&table) {
                    #(
                        let cell = #root::Table::get_cell(&table, row, #idx);

                        if cell.is_none() {
                            return ::std::result::Result::Err(
                                ::std::concat!(
                                    "Cell in column ",
                                    ::std::stringify!(#idx),
                                    "/",
                                    ::std::stringify!(#variant),
                                    " is missing",
                                )
                            );
                        }

                        if !cell.unwrap().#is_ty() {
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
                    )*
                }

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
    derive: Option<&PathList>,
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

    let derive_attr = derive
        .filter(|list| !list.is_empty())
        .map(|list| quote!(#[derive(#(#list),*)]));

    let item_enum = parse_quote! {
        #[automatically_derived]
        #derive_attr
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
