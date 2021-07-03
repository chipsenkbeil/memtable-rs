pub mod cell;
pub mod column;
pub mod column_by_name;
pub mod insert_row;
pub mod into_column;
pub mod into_column_by_name;
pub mod mut_cell;
pub mod new;
pub mod pop_row;
pub mod push_row;
pub mod remove_row;
pub mod replace_cell;
pub mod row;
pub mod rows;

use super::{utils, TableColumn, TableMode};
use darling::ast::Style;
use quote::format_ident;
use syn::{Ident, ItemFn, Path, Type};

pub fn make_cell_fns(
    root: &Path,
    style: Style,
    table_data_name: &Ident,
    columns: &[&TableColumn],
) -> Vec<ItemFn> {
    make_many(
        style,
        columns,
        |name| format_ident!("{}", name),
        |args| {
            let ManyArgs {
                method_name,
                idx,
                variant_ty,
                as_variant,
                ..
            } = args;

            cell::make(cell::Args {
                root,
                idx,
                method_name: &method_name,
                variant_ty,
                table_data_name,
                as_variant,
            })
        },
    )
}

pub fn make_mut_cell_fns(
    root: &Path,
    style: Style,
    table_data_name: &Ident,
    columns: &[&TableColumn],
) -> Vec<ItemFn> {
    make_many(
        style,
        columns,
        |name| format_ident!("mut{}{}", u(style), name),
        |args| {
            let ManyArgs {
                method_name,
                idx,
                variant_ty,
                as_mut_variant,
                ..
            } = args;

            mut_cell::make(mut_cell::Args {
                root,
                idx,
                method_name: &method_name,
                variant_ty,
                table_data_name,
                as_mut_variant,
            })
        },
    )
}

pub fn make_column_fns(
    root: &Path,
    style: Style,
    table_data_name: &Ident,
    columns: &[&TableColumn],
) -> Vec<ItemFn> {
    make_many(
        style,
        columns,
        |name| format_ident!("{}_column", name),
        |args| {
            let ManyArgs {
                method_name,
                idx,
                variant_ty,
                as_variant,
                ..
            } = args;

            column::make(column::Args {
                root,
                idx,
                method_name: &method_name,
                variant_ty,
                table_data_name,
                as_variant,
            })
        },
    )
}

pub fn make_into_column_fns(
    root: &Path,
    style: Style,
    table_data_name: &Ident,
    columns: &[&TableColumn],
) -> Vec<ItemFn> {
    make_many(
        style,
        columns,
        |name| format_ident!("into{}{}_column", u(style), name),
        |args| {
            let ManyArgs {
                method_name,
                idx,
                variant_ty,
                into_variant,
                ..
            } = args;

            into_column::make(into_column::Args {
                root,
                idx,
                method_name: &method_name,
                variant_ty,
                table_data_name,
                into_variant,
            })
        },
    )
}

pub fn make_replace_cell_fns(
    root: &Path,
    style: Style,
    table_data_name: &Ident,
    columns: &[&TableColumn],
) -> Vec<ItemFn> {
    make_many(
        style,
        columns,
        |name| format_ident!("replace{}{}", u(style), name),
        |args| {
            let ManyArgs {
                method_name,
                idx,
                variant_ty,
                into_variant,
                variant,
                ..
            } = args;

            replace_cell::make(replace_cell::Args {
                root,
                idx,
                method_name: &method_name,
                variant_ty,
                table_data_name,
                variant,
                into_variant,
            })
        },
    )
}

struct ManyArgs<'a> {
    pub method_name: &'a Ident,
    pub idx: &'a syn::Index,
    pub variant_ty: &'a Type,
    pub as_variant: &'a Ident,
    pub as_mut_variant: &'a Ident,
    pub into_variant: &'a Ident,
    pub variant: &'a Ident,
}

fn make_many(
    style: Style,
    columns: &[&TableColumn],
    mut make_method_name: impl FnMut(&Ident) -> Ident,
    mut make_fn: impl FnMut(ManyArgs) -> ItemFn,
) -> Vec<ItemFn> {
    let cnt = columns.len();
    let idx = utils::make_column_indexes(columns);
    let variant_tys = utils::make_variant_types(columns);
    let utils::VariantMethodIdents {
        as_variant,
        as_mut_variant,
        into_variant,
        ..
    } = utils::make_variant_method_idents(style, columns);
    let snake_idents = utils::make_snake_idents(columns);
    let variants = utils::make_variant_idents(columns);

    let mut fns = Vec::new();
    for i in 0..cnt {
        let args = ManyArgs {
            method_name: &make_method_name(&snake_idents[i]),
            idx: &idx[i],
            variant_ty: &variant_tys[i],
            as_variant: &as_variant[i],
            as_mut_variant: &as_mut_variant[i],
            into_variant: &into_variant[i],
            variant: &variants[i],
        };

        fns.push(make_fn(args));
    }
    fns
}

/// Returns _ if style is not a tuple struct (so no extra _ prefix)
fn u(style: Style) -> &'static str {
    if !style.is_tuple() {
        "_"
    } else {
        ""
    }
}
