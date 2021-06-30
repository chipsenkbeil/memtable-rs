use darling::{
    ast,
    util::{PathList, SpannedValue},
    FromDeriveInput, FromField, FromMeta,
};
use quote::format_ident;
use syn::{Generics, Ident, Type, Visibility};

/// Information about a table's Rust struct
#[derive(Debug, FromDeriveInput)]
#[darling(
    attributes(table),
    supports(struct_named, struct_newtype, struct_tuple)
)]
pub struct StructTable {
    pub ident: Ident,
    pub vis: Visibility,
    pub generics: Generics,
    pub data: ast::Data<(), TableColumn>,

    /// If provided, name to use for table instead of struct name
    #[darling(default)]
    pub name: Option<String>,

    /// If provided, will skip implementing From<...> for going to/from
    /// individual fields to the struct where derive was defined
    #[darling(default)]
    pub skip_parts: Option<SpannedValue<()>>,

    /// Attributes within data(...)
    #[darling(default, rename = "data")]
    pub data_attr: Option<TableDataAttr>,

    /// Derives to forward to derived table
    #[darling(default)]
    pub derive: Option<PathList>,

    /// Mode to use when generating the table
    #[darling(default)]
    pub mode: TableMode,
}

impl StructTable {
    pub fn as_style(&self) -> darling::ast::Style {
        self.data
            .as_ref()
            .take_struct()
            .map(|fields| fields.style)
            .expect("BUG: Identified struct but data missing!")
    }

    pub fn to_table_name(&self) -> Ident {
        self.name
            .as_ref()
            .map(|x| format_ident!("{}", x))
            .unwrap_or_else(|| format_ident!("{}Table", &self.ident))
    }

    pub fn to_table_data_name(&self) -> Ident {
        self.data_attr
            .as_ref()
            .and_then(|x| x.name.as_ref())
            .map(|x| format_ident!("{}", x))
            .unwrap_or_else(|| format_ident!("{}TableData", &self.ident))
    }

    pub fn columns(&self) -> Vec<&TableColumn> {
        let x = self.data.as_ref().take_struct();
        x.unwrap().fields
    }
}

/// Information for a data(...) attribute
#[derive(Debug, Default, FromMeta)]
#[darling(default)]
pub struct TableDataAttr {
    /// If provided, name to use for table's data instead of struct name
    #[darling(default)]
    pub name: Option<String>,

    /// Derives to forward to derived table data
    #[darling(default)]
    pub derive: Option<PathList>,
}

/// Represents the mode to use when generating code for a table
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromMeta)]
#[darling(rename_all = "snake_case")]
pub enum TableMode {
    /// Generated table leverages a dynamic table underneath
    Dynamic,

    /// Generated table leverages a fixed-column table underneath, where the
    /// column count matches the total number of fields from the struct
    FixedColumn,

    /// Generated table leverages a fixed row & column table underneath, where
    /// the column count matches the total number of fields from the struct
    /// and the row count is specified manually by the end user
    Fixed { rows: usize },
}

impl Default for TableMode {
    /// Defaults to dynamic, which doesn't require the fields to implement default
    fn default() -> Self {
        Self::Dynamic
    }
}

/// Information for a field of a struct deriving table
#[derive(Debug, FromField)]
#[darling(attributes(column))]
pub struct TableColumn {
    /// Name of the column field
    pub ident: Option<Ident>,

    /// Type of the column
    pub ty: Type,

    /// If provided, flags column to be indexed
    #[darling(default)]
    pub indexed: Option<SpannedValue<()>>,

    /// If provided, name to use for column instead of its field name
    #[darling(default)]
    pub name: Option<String>,
}
