use super::{TableColumn, TableMode};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_quote, Generics, Ident, LitInt, LitStr, Path, Type};
use voca_rs::case;

pub fn make_inner_table_type(
    root: &Path,
    mode: TableMode,
    table_data_name: &Ident,
    generics: &Generics,
    col_cnt: usize,
) -> Type {
    let (_, ty_generics, _) = generics.split_for_impl();
    let col_cnt: LitInt = parse_quote!(#col_cnt);

    match mode {
        TableMode::Dynamic => {
            parse_quote!(#root::DynamicTable<#table_data_name #ty_generics>)
        }
        TableMode::FixedColumn => {
            parse_quote!(#root::FixedColumnTable<#table_data_name #ty_generics, #col_cnt>)
        }
        TableMode::Fixed { rows } => {
            let row_cnt: LitInt = parse_quote!(#rows);
            parse_quote!(#root::FixedTable<#table_data_name #ty_generics, #row_cnt, #col_cnt>)
        }
    }
}

pub fn make_variant_idents(columns: &[&TableColumn]) -> Vec<Ident> {
    columns
        .iter()
        .enumerate()
        .map(|(idx, col)| {
            let name = if let Some(name) = col.name.as_ref() {
                case::pascal_case(&name)
            } else if let Some(name) = col.ident.as_ref() {
                case::pascal_case(&name.to_string())
            } else {
                format!("_{}", idx)
            };
            format_ident!("{}", name)
        })
        .collect()
}

pub fn make_variant_types(columns: &[&TableColumn]) -> Vec<Type> {
    columns.iter().map(|col| col.ty.clone()).collect()
}

pub fn make_column_indexes(columns: &[&TableColumn]) -> Vec<syn::Index> {
    (0..columns.len()).map(syn::Index::from).collect()
}

pub fn make_snake_idents(columns: &[&TableColumn]) -> Vec<Ident> {
    columns
        .iter()
        .enumerate()
        .map(|(idx, col)| {
            let name = if let Some(name) = col.name.as_ref() {
                case::snake_case(&name)
            } else if let Some(name) = col.ident.as_ref() {
                case::snake_case(&name.to_string())
            } else {
                format!("_{}", idx)
            };
            format_ident!("{}", name)
        })
        .collect()
}

pub fn make_field_tokens(columns: &[&TableColumn]) -> Vec<TokenStream> {
    columns
        .iter()
        .enumerate()
        .map(|(idx, col)| {
            col.ident.as_ref().map(|x| quote!(#x)).unwrap_or_else(|| {
                let idx = syn::Index::from(idx);
                quote!(#idx)
            })
        })
        .collect()
}

pub struct VariantMethodIdents {
    pub is_variant: Vec<Ident>,
    pub as_variant: Vec<Ident>,
    pub as_mut_variant: Vec<Ident>,
    pub into_variant: Vec<Ident>,
}

pub fn make_variant_method_idents(columns: &[&TableColumn]) -> VariantMethodIdents {
    let method_names: Vec<(Ident, Ident, Ident, Ident)> = make_snake_idents(columns)
        .into_iter()
        .map(|suffix| {
            (
                format_ident!("is_{}", suffix),
                format_ident!("as_{}", suffix),
                format_ident!("as_mut_{}", suffix),
                format_ident!("into_{}", suffix),
            )
        })
        .collect();
    let (is_variant, as_variant, as_mut_variant, into_variant) = method_names.into_iter().fold(
        (Vec::new(), Vec::new(), Vec::new(), Vec::new()),
        |(mut is_acc, mut as_acc, mut as_mut_acc, mut into_acc),
         (is_name, as_name, as_mut_name, into_name)| {
            is_acc.push(is_name);
            as_acc.push(as_name);
            as_mut_acc.push(as_mut_name);
            into_acc.push(into_name);
            (is_acc, as_acc, as_mut_acc, into_acc)
        },
    );

    VariantMethodIdents {
        is_variant,
        as_variant,
        as_mut_variant,
        into_variant,
    }
}

pub fn make_column_names(
    columns: &[&TableColumn],
    map_name: impl Fn(&str) -> String,
) -> Vec<String> {
    columns
        .iter()
        .enumerate()
        .map(|(idx, col)| {
            if let Some(name) = col.name.as_ref() {
                map_name(&name)
            } else if let Some(name) = col.ident.as_ref() {
                map_name(&name.to_string())
            } else {
                idx.to_string()
            }
        })
        .collect()
}

#[inline]
pub fn bug_str() -> LitStr {
    let msg = concat!(
        "BUG: Typed row missing cell data! This should never happen! ",
        "Please report this to https://github.com/chipsenkbeil/memtable-rs/issues",
    );

    parse_quote!(#msg)
}
