pub mod column;
pub mod column_by_name;
pub mod column_names;
pub mod get_cell;
pub mod get_mut_cell;
pub mod insert_row;
pub mod into_column;
pub mod into_column_by_name;
pub mod new;
pub mod pop_row;
pub mod push_row;
pub mod remove_row;
pub mod replace_cell;
pub mod row;
pub mod rows;

use super::{utils, TableColumn, TableMode};
use quote::format_ident;
use syn::{Ident, ItemFn, Path, Type};

pub fn make_get_cell_fns(
    root: &Path,
    mode: TableMode,
    table_data_name: &Ident,
    columns: &[&TableColumn],
) -> Vec<ItemFn> {
    make_many(
        columns,
        |name| format_ident!("get_{}", name),
        |args| {
            let ManyArgs {
                method_name,
                idx,
                variant_ty,
                as_variant,
                into_variant,
                ..
            } = args;

            get_cell::make(get_cell::Args {
                root,
                mode,
                idx,
                method_name: &method_name,
                variant_ty,
                table_data_name,
                as_variant,
                into_variant,
            })
        },
    )
}

pub fn make_get_mut_cell_fns(
    root: &Path,
    mode: TableMode,
    table_data_name: &Ident,
    columns: &[&TableColumn],
) -> Vec<ItemFn> {
    make_many(
        columns,
        |name| format_ident!("get_mut_{}", name),
        |args| {
            let ManyArgs {
                method_name,
                idx,
                variant_ty,
                as_mut_variant,
                into_variant,
                ..
            } = args;

            get_mut_cell::make(get_mut_cell::Args {
                root,
                mode,
                idx,
                method_name: &method_name,
                variant_ty,
                table_data_name,
                as_mut_variant,
                into_variant,
            })
        },
    )
}

pub fn make_column_fns(
    root: &Path,
    mode: TableMode,
    table_data_name: &Ident,
    columns: &[&TableColumn],
) -> Vec<ItemFn> {
    make_many(
        columns,
        |name| format_ident!("{}_column", name),
        |args| {
            let ManyArgs {
                method_name,
                idx,
                variant_ty,
                as_variant,
                into_variant,
                ..
            } = args;

            column::make(column::Args {
                root,
                mode,
                idx,
                method_name: &method_name,
                variant_ty,
                table_data_name,
                as_variant,
                into_variant,
            })
        },
    )
}

pub fn make_into_column_fns(
    root: &Path,
    table_data_name: &Ident,
    columns: &[&TableColumn],
) -> Vec<ItemFn> {
    make_many(
        columns,
        |name| format_ident!("into_{}_column", name),
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
    table_data_name: &Ident,
    columns: &[&TableColumn],
) -> Vec<ItemFn> {
    make_many(
        columns,
        |name| format_ident!("replace_{}", name),
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
    } = utils::make_variant_method_idents(columns);
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
