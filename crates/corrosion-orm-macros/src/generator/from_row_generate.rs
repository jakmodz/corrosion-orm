use crate::{
    TableData,
    model::{Field, primary_key::PrimaryKeyField},
};
use corrosion_orm_core::schema::relation::RelationType;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;

/// Generate a Rust `TokenStream` that implements `sqlx::FromRow` for the provided table struct.
///
/// The generated implementation constructs the struct by reading the primary key and each
/// non-relation field from a SQL row using `row.try_get(...)`, and initializes relation fields
/// as follows: for `BelongsTo` relations the related entity is default-constructed and its ID
/// is set from the row's foreign-key column; for other relations the field is set to `Default`.
///
/// Parameters:
/// - `table`: metadata describing the target struct (identifier, primary key, fields, and relations).
///
/// # Examples
///
/// ```no_run
/// // Given a `table: TableData` describing a struct, generate the FromRow impl:
/// let tokens = generate_from_row(&table);
/// // `tokens` now contains the `impl<'r, R: sqlx::Row> sqlx::FromRow<'r, R> for ...` block.
/// ```
pub(crate) fn generate_from_row(table: &TableData) -> TokenStream {
    let struct_ident = &table.ident;

    let pk_field_assign = generate_pk_field_assign(&table.primary_key);
    // Note: table.fields already excludes the PK and relation fields (they are parsed separately)
    let field_assigns: Vec<TokenStream> = table.fields.iter().map(generate_field_assign).collect();

    let relation_field_assigns: Vec<TokenStream> = table
        .relations
        .iter()
        .map(|rel| {
            let field_name = syn::Ident::new(&rel.relation_name, proc_macro2::Span::call_site());
            let rel_ty = &rel.ty;
            let fk_name = &rel.foreign_key;
            match rel.relation_type {
                RelationType::BelongsTo => {
                    // For BelongsTo, read the FK column from the row and populate the
                    // related entity's primary key so that load_relations can fetch it.
                    quote! {
                        #field_name: {
                            let mut rel = #rel_ty::default();
                            rel.set_id(row.try_get(#fk_name)?);
                            rel
                        },
                    }
                }
                _ => {
                    quote! {
                        #field_name: Default::default(),
                    }
                }
            }
        })
        .collect();
    let pk_bound = type_bounds(&table.primary_key.ty);
    let field_bounds: Vec<TokenStream> = table.fields.iter().map(|f| type_bounds(&f.ty)).collect();

    quote! {
        impl<'r, R: sqlx::Row> sqlx::FromRow<'r, R> for #struct_ident
        where
            for<'c> &'c str: sqlx::ColumnIndex<R>,
            #pk_bound
            #(#field_bounds)*
        {
            fn from_row(row: &'r R) -> sqlx::Result<Self> {
                Ok(Self {
                    #pk_field_assign
                    #(#field_assigns)*
                    #(#relation_field_assigns)*
                })
            }
        }
    }
}

fn type_bounds(ty: &Type) -> TokenStream {
    quote! {
        #ty: sqlx::decode::Decode<'r, R::Database>,
        #ty: sqlx::types::Type<R::Database>,
    }
}

fn generate_pk_field_assign(pk: &PrimaryKeyField) -> TokenStream {
    let field_ident = &pk.iden;
    let col_name = &pk.name;
    quote! {
        #field_ident: row.try_get(#col_name)?,
    }
}

fn generate_field_assign(field: &Field) -> TokenStream {
    let field_ident = &field.iden;
    let col_name = &field.name;
    quote! {
        #field_ident: row.try_get(#col_name)?,
    }
}
