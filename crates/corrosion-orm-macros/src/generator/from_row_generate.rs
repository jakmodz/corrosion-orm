use crate::{
    TableData,
    model::{Field, primary_key::PrimaryKeyField},
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;

pub(crate) fn generate_from_row(table: &TableData) -> TokenStream {
    let struct_ident = &table.ident;

    let pk_field_assign = generate_pk_field_assign(&table.primary_key);
    let field_assigns: Vec<TokenStream> = table.fields.iter().map(generate_field_assign).collect();

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
