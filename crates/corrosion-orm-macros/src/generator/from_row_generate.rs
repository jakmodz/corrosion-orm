use crate::utils::{extract_inner_type, extract_type_ident, is_option_type};
use crate::{
    TableData,
    model::{Field, primary_key::PrimaryKeyField},
};
use corrosion_orm_core::schema::relation::RelationType;
use proc_macro2::TokenStream;
use quote::quote;
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
/// ```ignore
/// // Given a `table: TableData` describing a struct, generate the FromRow impl:
/// let tokens = generate_from_row(&table);
/// // `tokens` now contains the `impl<'r, R: sqlx::Row> sqlx::FromRow<'r, R> for ...` block.
/// ```
pub(crate) fn generate_from_row(table: &TableData) -> TokenStream {
    let struct_ident = &table.ident;
    let owner_pk_name = &table.primary_key.name;
    let owner_pk_ty = &table.primary_key.ty;
    let orm = super::orm_crate_path();

    let pk_field_assign = generate_pk_field_assign(&orm, &table.primary_key);
    let field_assigns: Vec<TokenStream> = table
        .fields
        .iter()
        .map(|f| generate_field_assign(&orm, f))
        .collect();

    let relation_field_assigns: Vec<TokenStream> = table
        .relations
        .iter()
        .map(|rel| {
            let field_name = syn::Ident::new(&rel.relation_name, proc_macro2::Span::call_site());
            let rel_ty = &rel.ty;
            let fk_name = &rel.foreign_key;
            let fk_column_ident = syn::Ident::new(fk_name, proc_macro2::Span::call_site());


            match rel.relation_type {
                RelationType::BelongsTo | RelationType::HasOne => {
                    if !rel.is_eager {
                        let related_ty = extract_inner_type(rel_ty);
                        quote! {
                            #field_name: {
                                let mut lazy: #rel_ty = #orm::model::lazy::Lazy::new();
                                let mut rel = <#related_ty>::default();
                                rel.set_id(row.try_get(#fk_name)?);

                                lazy.set_condition(
                                    #orm::model::lazy::LazyCondition::ById(
                                            #orm::query::query_type::Value::from(rel.get_id())
                                        )
                                    );
                                lazy
                            },
                        }
                    } else {
                        quote! {
                            #field_name: {
                                let mut rel = #rel_ty::default();
                                rel.set_id(row.try_get(#fk_name)?);
                                rel
                            },
                        }
                    }
                }

                RelationType::HasMany => {
                    if !rel.is_eager {
                        let child_ty = extract_inner_type(rel_ty);
                        let child_ident = extract_type_ident(&child_ty)
                            .expect("HasMany relation inner type must be a path type");
                        let child_mod = syn::Ident::new(
                            &child_ident.to_string().to_lowercase(),
                            proc_macro2::Span::call_site(),
                        );
                        quote! {
                            #field_name: {
                                let mut lazy: #rel_ty = #orm::model::lazy_collection::LazyCollection::new();
                                let owner_id: #owner_pk_ty = row.try_get(#owner_pk_name)?;
                                lazy.set_condition(
                                    corrosion_orm_core::model::lazy_collection::LazyCollectionCondition::ByForeignKey {
                                        fk_column: #child_mod::Column::#fk_column_ident,
                                        value: #orm::query::query_type::Value::from(owner_id),
                                    }
                                );
                                lazy
                            },
                        }
                    } else {
                        quote! { #field_name: Default::default(), }
                    }
                }

                _ => quote! {
                    #field_name: Default::default(),
                },
            }
        })
        .collect();

    quote! {
        impl #orm::driver::from_row_db::FromRowDb for #struct_ident {
            fn from_row(row: &#orm::driver::db_row::DbRow) -> Result<Self, #orm::error::CorrosionOrmError> {
                Ok(Self {
                    #pk_field_assign
                    #(#field_assigns)*
                    #(#relation_field_assigns)*
                })
            }
        }
    }
}

fn generate_pk_field_assign(_orm: &TokenStream, pk: &PrimaryKeyField) -> TokenStream {
    let field_ident = &pk.iden;
    let col_name = &pk.name;
    let ty = &pk.ty;
    quote! {
        #field_ident: row.try_get::<#ty>(#col_name)?,
    }
}

fn generate_field_assign(_orm: &TokenStream, field: &Field) -> TokenStream {
    let field_ident = &field.iden;
    let col_name = &field.name;
    if field.is_nullable {
        if is_option_type(&field.ty) {
            let inner_ty = extract_inner_type(&field.ty);
            quote! {
                #field_ident: row.try_get_optional::<#inner_ty>(#col_name)?,
            }
        } else {
            let ty = &field.ty;
            quote! {
                #field_ident: row.try_get_optional::<#ty>(#col_name)?.unwrap_or_default(),
            }
        }
    } else {
        let ty = &field.ty;
        quote! {
            #field_ident: row.try_get::<#ty>(#col_name)?,
        }
    }
}
