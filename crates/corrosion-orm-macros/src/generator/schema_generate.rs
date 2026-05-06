use crate::TableData;
use crate::model::Field;
use crate::model::primary_key::PrimaryKeyField;
use crate::model::relation::RelationDefinition;
use corrosion_orm_core::schema::relation::RelationType;
use corrosion_orm_core::types::generation_strategy::GenerationType;
use proc_macro2::TokenStream;
use quote::quote;

fn extract_vec_inner_type(ty: &syn::Type) -> syn::Type {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
        && segment.ident == "Vec"
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
    {
        return inner_ty.clone();
    }

    ty.clone()
}

pub fn generate_schema_impl(table: &TableData) -> TokenStream {
    let mut fields: Vec<TokenStream> = Vec::new();
    for field in table.fields.iter() {
        fields.push(generate_field(field));
    }

    let mut indexes: Vec<TokenStream> = Vec::new();
    for index in table.indexes.iter() {
        indexes.push(generate_index(index));
    }
    let mut checks: Vec<TokenStream> = Vec::new();
    let mut relations: Vec<TokenStream> = Vec::new();
    let struct_ident = &table.ident;
    for (relation_index, relation) in table.relations.iter().enumerate() {
        let (check, relation) =
            generate_relation(relation, &table.name, relation_index, struct_ident);
        checks.push(check);
        relations.push(relation);
    }

    let table_name = &table.name;

    let primary_key = generate_primary_key(&table.primary_key);
    quote! {
        #(#checks)*
        impl corrosion_orm_core::schema::table::TableSchema for #struct_ident{
            fn get_table_name()->&'static str{
                #table_name
            }
            fn get_schema()->corrosion_orm_core::schema::table::TableSchemaModel{
                corrosion_orm_core::schema::table::TableSchemaModel{
                    name: String::from(#table_name),
                    fields: vec!(#(#fields),*),
                    indexes: vec!(#(#indexes),*),
                    primary_key: #primary_key,
                    relations: vec!(#(#relations),*)
                }
            }
        }
    }
}

fn generate_field(field: &Field) -> TokenStream {
    let field_name = &field.name;
    let field_type = &field.ty;
    let is_nullable = field.is_nullable;
    let is_unique = field.is_unique;
    let sql_ty = match &field.column_definition {
        Some(ty) => quote! {
            corrosion_orm_core::types::column_type::SqlType::Custom(String::from(#ty))
        },
        None => quote! {
            <#field_type as corrosion_orm_core::types::column_type::ToSqlType>::to_sql_type(
                &<#field_type>::default()
            )
        },
    };
    quote! {
        corrosion_orm_core::schema::table::ColumnSchemaModel::new::<#field_type>(
            String::from(#field_name),
            #is_nullable,
            #is_unique,
            #sql_ty
        )
    }
}

fn generate_index(index: &crate::model::IndexDefinition) -> TokenStream {
    let index_name = &index.name;
    let index_fields: Vec<String> = index.fields.clone();
    let is_unique = index.unique;

    quote! {
        corrosion_orm_core::schema::table::IndexModel {
            name: String::from(#index_name),
            fields: vec!(#(String::from(#index_fields)),*),
            unique: #is_unique,
        }
    }
}

fn generate_primary_key(primary_key: &PrimaryKeyField) -> TokenStream {
    let key_name = &primary_key.name;
    let strategy = match &primary_key.generation_strategy {
        Some(GenerationType::AutoIncrement) => quote! {
            Some(corrosion_orm_core::types::generation_strategy::GenerationType::AutoIncrement)
        },
        None => quote! { None },
    };
    let field_type = &primary_key.ty;
    quote! {
        corrosion_orm_core::schema::table::PrimaryKeyModel::new(
            String::from(#key_name),
            #strategy,
            <#field_type>::default()
        )
    }
}
fn generate_relation(
    relation: &RelationDefinition,
    source_table: &str,
    relation_index: usize,
    struct_ident: &syn::Ident,
) -> (TokenStream, TokenStream) {
    let ty = match &relation.relation_type {
        RelationType::HasOne => quote! {corrosion_orm_core::schema::relation::RelationType::HasOne},
        RelationType::HasMany => {
            quote! {corrosion_orm_core::schema::relation::RelationType::HasMany}
        }
        RelationType::BelongsTo => {
            quote! {corrosion_orm_core::schema::relation::RelationType::BelongsTo}
        }
        RelationType::BelongsToMany => {
            quote! {corrosion_orm_core::schema::relation::RelationType::BelongsToMany}
        }
    };

    let key = &relation.foreign_key;
    let relation_name = &relation.relation_name;
    let field_type = &relation.ty;
    let is_unique = match &relation.relation_type {
        RelationType::HasOne => true,
        RelationType::HasMany | RelationType::BelongsTo | RelationType::BelongsToMany => false,
    };

    let check_type = if matches!(
        relation.relation_type,
        RelationType::HasMany | RelationType::BelongsToMany
    ) {
        extract_vec_inner_type(field_type)
    } else {
        field_type.clone()
    };

    let table_expr_type = if matches!(
        relation.relation_type,
        RelationType::HasMany | RelationType::BelongsToMany
    ) {
        &check_type
    } else {
        field_type
    };

    let table_expr = match &relation.table {
        Some(t) => quote! { String::from(#t) },
        None => {
            quote! { <#table_expr_type as corrosion_orm_core::schema::table::TableSchema>::get_table_name().to_string() }
        }
    };

    let trait_name = syn::Ident::new(
        &format!("MustDeriveModel_{}_{}", struct_ident, relation_index),
        proc_macro2::Span::call_site(),
    );

    let check = quote! {
        #[diagnostic::on_unimplemented(
            message = "`{Self}` must derive `Model` to be used in a relation",
            label = "this type does not derive `Model`",
            note = "add `#[derive(Model)]` to `{Self}`"
        )]
        trait #trait_name: corrosion_orm_core::schema::table::TableSchema {}
        impl<T: corrosion_orm_core::schema::table::TableSchema> #trait_name for T {}

        const _: fn() = || {
            fn check_relation_target<T: #trait_name>() {}
            check_relation_target::<#check_type>();
        };
    };
    let relation = quote! {
        corrosion_orm_core::schema::relation::RelationModel::new(
            #ty,
            #table_expr,
            String::from(#key),
            <#check_type as corrosion_orm_core::schema::table::TableSchema>::get_schema().primary_key.name,
            String::from(#relation_name),
            String::from(#source_table),
            corrosion_orm_core::schema::table::ColumnSchemaModel {
                name: String::from(#key),
                is_nullable: false,
                is_unique: #is_unique,
                sql_type: <#check_type as corrosion_orm_core::schema::table::TableSchema>::get_schema().primary_key.ty
            },
        )
    };
    (check, relation)
}
