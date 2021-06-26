use darling::{
    ast,
    util::{PathList, SpannedValue},
    FromDeriveInput, FromField, FromMeta,
};
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

/// Information for a field of a struct deriving table
#[derive(Debug, FromField)]
#[darling(attributes(table))]
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
