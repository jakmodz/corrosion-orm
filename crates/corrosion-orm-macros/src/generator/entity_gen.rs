use quote::quote;
use syn::Type;

use crate::TableData;

use corrosion_orm_core::types::column_type::{SqlType, ToSqlType};

/// Maps SqlType (your IR) to column wrapper types
fn sql_type_to_wrapper(sql_type: &SqlType) -> proc_macro2::TokenStream {
    match sql_type {
        SqlType::Varchar(_) | SqlType::Text | SqlType::Char(_) => {
            quote! { corrosion_orm_core::types::StringColumn }
        }

        SqlType::Integer | SqlType::Float | SqlType::Double => {
            quote! { corrosion_orm_core::types::NumericColumn }
        }

        SqlType::Boolean => {
            quote! { corrosion_orm_core::types::BooleanColumn }
        }

        SqlType::Date | SqlType::Timestamp => {
            quote! { corrosion_orm_core::types::DateLikeColumn }
        }

        SqlType::Custom(name) => {
            let name_lower = name.to_lowercase();
            if name_lower.contains("date") || name_lower.contains("time") {
                quote! { corrosion_orm_core::types::DateLikeColumn }
            } else if name_lower.contains("int")
                || name_lower.contains("numeric")
                || name_lower.contains("float")
            {
                quote! { corrosion_orm_core::types::NumericColumn }
            } else {
                quote! { corrosion_orm_core::types::StringColumn }
            }
        }
    }
}

/// Map a Rust AST type to the corresponding `SqlType`.
///
/// This inspects the provided `syn::Type` and returns a best-effort `SqlType`:
/// - If the type is `Option<T>`, the mapping is performed on `T`.
/// - Common Rust types are mapped to their expected SQL equivalents (e.g. `String` → string type, `i32`/`f64`/`bool` → numeric/boolean types, `NaiveDate`/`NaiveDateTime` → date/timestamp).
/// - An unrecognized type path identifier yields `SqlType::Custom(<identifier>)`.
/// - Non-path types or missing identifiers yield `SqlType::Custom("Unknown".to_string())`.
///
/// # Examples
///
/// ```ignore
/// use syn::Type;
///
/// let t = syn::parse_str::<Type>("i32").unwrap();
/// assert_eq!(get_sql_type_from_rust_type(&t), i32::default().to_sql_type());
///
/// let opt_t = syn::parse_str::<Type>("Option<String>").unwrap();
/// assert_eq!(get_sql_type_from_rust_type(&opt_t), String::default().to_sql_type());
/// ```
fn get_sql_type_from_rust_type(ty: &Type) -> SqlType {
    match ty {
        syn::Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                if segment.ident == "Option"
                    && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
                    && let syn::GenericArgument::Type(inner_ty) = args.args.first().unwrap()
                {
                    return get_sql_type_from_rust_type(inner_ty);
                }

                let ident_str = segment.ident.to_string();
                match ident_str.as_str() {
                    "String" => String::default().to_sql_type(),
                    "NaiveDate" => SqlType::Date,
                    "NaiveDateTime" => SqlType::Timestamp,
                    "bool" => bool::default().to_sql_type(),
                    "i32" => i32::default().to_sql_type(),
                    "i64" => i64::default().to_sql_type(),
                    "u32" => u32::default().to_sql_type(),
                    "u64" => u64::default().to_sql_type(),
                    "i16" => i16::default().to_sql_type(),
                    "u16" => u16::default().to_sql_type(),
                    "i8" => i8::default().to_sql_type(),
                    "u8" => u8::default().to_sql_type(),
                    "f32" => f32::default().to_sql_type(),
                    "f64" => f64::default().to_sql_type(),
                    _ => SqlType::Custom(ident_str),
                }
            } else {
                SqlType::Custom("Unknown".to_string())
            }
        }
        _ => SqlType::Custom("Unknown".to_string()),
    }
}

/// Generates a Rust module (as a TokenStream) that defines a typed column enum and a `Columns` value for an entity.
///
/// The generated module is named from `table.ident` (lowercased) and contains:
/// - A `Column` enum with one variant per column (derived `Debug, Clone, Copy, PartialEq, Eq`).
/// - An implementation of `corrosion_orm_core::types::ColumnTrait` for `Column` that returns the table name
///   (via the entity struct's `TableSchema::get_table_name`) and the column name for each variant.
/// - A `Columns` struct with a field for each column, typed as the appropriate column wrapper (e.g., `StringColumn`, `NumericColumn`, etc.)
///   and a `pub const COLUMN: Columns` initializer that constructs each wrapper with the corresponding `Column` variant.
///   If the table has no columns, an empty `Column` enum and an empty `Columns` struct with a `COLUMN` value are generated.
///
/// # Parameters
///
/// table — Source `TableData` describing the entity name (`ident`), the primary key (`primary_key`) and other fields (`fields`).
///
/// # Returns
///
/// A `proc_macro2::TokenStream` containing the generated module definition for the entity.
pub(crate) fn generate_entity(table: &TableData) -> proc_macro2::TokenStream {
    let module_ident = syn::Ident::new(
        &table.ident.to_string().to_lowercase(),
        proc_macro2::Span::call_site(),
    );
    let struct_ident = &table.ident;

    let mut column_defs = Vec::new();
    let primary_key_field = &table.primary_key;
    let field_name_lower = syn::Ident::new(
        &primary_key_field.name.to_lowercase(),
        proc_macro2::Span::call_site(),
    );
    let column_name = &primary_key_field.name;

    let sql_type = get_sql_type_from_rust_type(&primary_key_field.ty);
    let wrapper_type = sql_type_to_wrapper(&sql_type);
    column_defs.push((field_name_lower, column_name.clone(), wrapper_type));
    for field in &table.fields {
        let field_name_lower =
            syn::Ident::new(&field.name.to_lowercase(), proc_macro2::Span::call_site());
        let column_name = &field.name;

        let sql_type = get_sql_type_from_rust_type(&field.ty);
        let wrapper_type = sql_type_to_wrapper(&sql_type);

        column_defs.push((field_name_lower, column_name.clone(), wrapper_type));
    }

    let struct_fields = column_defs.iter().map(|(field_name, _, wrapper_type)| {
        quote! { pub #field_name: #wrapper_type<Column> }
    });

    let const_inits = column_defs
        .iter()
        .map(|(field_name, column_name_str, wrapper_type)| {
            let variant_ident = syn::Ident::new(column_name_str, proc_macro2::Span::call_site());
            quote! { #field_name: #wrapper_type::new(Column::#variant_ident) }
        });

    let columns_struct = if !column_defs.is_empty() {
        quote! {
            pub struct Columns {
                #(#struct_fields),*
            }

            pub const COLUMN: Columns = Columns {
                #(#const_inits),*
            };
        }
    } else {
        quote! {
            pub struct Columns;
            pub const COLUMN: Columns = Columns;
        }
    };

    let column_enum_def = if !column_defs.is_empty() {
        let variants = column_defs.iter().map(|(_, column_name, _)| {
            let variant_ident = syn::Ident::new(column_name, proc_macro2::Span::call_site());
            quote! {
                #variant_ident
            }
        });
        let variants = variants.collect::<Vec<_>>();
        let column_names = column_defs
            .iter()
            .map(|(_, column_name, _)| column_name)
            .collect::<Vec<_>>();

        let columns_enum_def = quote! {
                #[derive(Debug, Clone, Copy, PartialEq, Eq)]
                #[allow(non_camel_case_types)]
                pub enum Column {
                    #(#variants),*
                }

                impl corrosion_orm_core::types::ColumnTrait for Column {
                    fn table_name(&self) -> &'static str {

                        <super::#struct_ident as corrosion_orm_core::schema::table::TableSchema>::get_table_name()
                    }

                    fn column_name(&self) -> &'static str {
                        match self {
                            #(Self::#variants => #column_names),*
                        }
                    }
                }
        };

        columns_enum_def
    } else {
        quote! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub enum Column {
            }
        }
    };

    quote! {
        pub mod #module_ident {
            #column_enum_def
            #columns_struct
        }
    }
}
