mod entity_gen;
mod from_row_generate;
mod insert_plan_generate;
mod repository_generate;
mod schema_generate;
pub mod validation_generate;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Ident, Span};
use quote::quote;

use crate::{
    TableData,
    generator::{
        entity_gen::generate_entity, from_row_generate::generate_from_row,
        insert_plan_generate::generate_insert_plan, repository_generate::generate_repository,
    },
};

pub(crate) fn orm_crate_path() -> proc_macro2::TokenStream {
    let found = crate_name("corrosion-orm")
        .or_else(|_| crate_name("corrosion-orm-core"))
        .expect("corrosion-orm or corrosion-orm-core must be a dependency");
    match found {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(::#ident)
        }
    }
}

pub(crate) fn generate_impl(table: &TableData) -> proc_macro::TokenStream {
    let schema_impl = schema_generate::generate_schema_impl(table);
    let from_row_impl = generate_from_row(table);
    let repository_impl = generate_repository(table);
    let entity_impl = generate_entity(table);
    let insert_plan_impl = generate_insert_plan(table);
    quote! {
        #schema_impl
        #entity_impl
        #from_row_impl
        #insert_plan_impl
        #repository_impl
    }
    .into()
}
