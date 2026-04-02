use crate::TableData;
use crate::model::Field;
use crate::model::primary_key::PrimaryKeyField;
use corrosion_orm_core::types::generation_strategy::GenerationType;
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_schema_impl(table: &TableData) -> TokenStream {
    let mut fields: Vec<TokenStream> = Vec::new();
    for field in table.fields.iter() {
        fields.push(generate_field(field));
    }

    let table_name = &table.name;
    let struct_ident = &table.ident;

    let primary_key = generate_primary_key(&table.primary_key);

    quote! {
        impl corrosion_orm_core::schema::table::TableSchema for #struct_ident{
            fn get_table_name()->&'static str{
                #table_name
            }
            fn get_schema()->corrosion_orm_core::schema::table::TableSchemaModel{
                corrosion_orm_core::schema::table::TableSchemaModel{
                    name: String::from(#table_name),
                    fields: vec!(#(#fields),*),
                    indexes: Vec::new(),
                    primary_key: #primary_key
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
