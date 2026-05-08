use crate::TableData;
use crate::model::Field;
use crate::model::primary_key::PrimaryKeyField;
use crate::model::relation::RelationDefinition;
use corrosion_orm_core::schema::relation::RelationType;
use corrosion_orm_core::types::generation_strategy::GenerationType;
use proc_macro2::TokenStream;
use quote::quote;

/// Extracts the inner type `T` from a `Vec<T>` `syn::Type`, or returns the original type unchanged.
///
/// If `ty` is a path type whose last segment is `Vec` with a single generic type argument, this
/// function returns a clone of that inner type. For any other `syn::Type` it returns a clone of
/// `ty`.
///
/// # Examples
///
/// ```ignore
/// use syn::Type;
/// use quote::ToTokens;
///
/// // Vec inner type is extracted
/// let vec_ty: Type = syn::parse_str("Vec<i32>").unwrap();
/// let inner = extract_vec_inner_type(&vec_ty);
/// assert_eq!(inner.to_token_stream().to_string(), "i32");
///
/// // Non-Vec types are returned unchanged (cloned)
/// let simple_ty: Type = syn::parse_str("String").unwrap();
/// let same = extract_vec_inner_type(&simple_ty);
/// assert_eq!(same.to_token_stream().to_string(), "String");
/// ```
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

/// Generate a TokenStream that implements the TableSchema trait for the provided table definition.
///
/// # Arguments
///
/// * `table` - Definition of a table (name, ident, fields, indexes, primary key and relations) used to build the schema implementation.
///
/// # Returns
///
/// A `TokenStream` containing an `impl corrosion_orm_core::schema::table::TableSchema` block for the table, plus any compile-time relation checks; the emitted schema includes the table name, fields, indexes, primary key and relations.
///
/// # Examples
///
/// ```ignore
/// // Given a `TableData` describing a struct, generate the schema tokens:
/// // let table: TableData = /* construct or obtain TableData */ ;
/// let tokens = generate_schema_impl(&table);
/// let s = tokens.to_string();
/// assert!(s.contains("impl corrosion_orm_core::schema::table::TableSchema"));
/// ```
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
/// Generates the token streams required for a relation: a compile-time trait check that enforces the related type derives `Model`, and a `RelationModel` construction for the relation.
///
/// The first returned token stream declares a helper trait with a custom on-unimplemented diagnostic and a const wrapper that forces a compile-time check that the relation target implements `TableSchema` (i.e., derives `Model`). The second token stream constructs a `corrosion_orm_core::schema::relation::RelationModel` using the relation metadata (relation type, target table expression, foreign key, primary key name of the related table, relation name, source table, and a `ColumnSchemaModel` describing the foreign key column).
///
/// Returns a tuple `(check_tokens, relation_tokens)` where `check_tokens` is the compile-time check TokenStream and `relation_tokens` is the RelationModel TokenStream.
///
/// # Examples
///
/// ```ignore
/// use proc_macro2::TokenStream;
/// use syn::Ident;
///
/// // `relation_def` should be a valid RelationDefinition value; this example shows calling the function.
/// // `generate_relation` returns (check_tokens, relation_tokens).
/// let relation_def = /* obtain or construct a RelationDefinition */ unimplemented!();
/// let struct_ident = Ident::new("MyStruct", proc_macro2::Span::call_site());
/// let (check_tokens, relation_tokens): (TokenStream, TokenStream) = generate_relation(&relation_def, "source_table", 0, &struct_ident);
/// ```
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
