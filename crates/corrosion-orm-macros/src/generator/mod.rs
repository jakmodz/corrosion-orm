mod from_row_generate;
mod repository_generate;
mod schema_generate;
use quote::quote;

use crate::{
    TableData,
    generator::{from_row_generate::generate_from_row, repository_generate::generate_repository},
};

pub(crate) fn generate_impl(table: &TableData) -> proc_macro::TokenStream {
    let schema_impl = schema_generate::generate_schema_impl(table);
    let from_row_impl = generate_from_row(table);
    let repository_impl = generate_repository(table);
    quote! {
        #schema_impl
        #from_row_impl
        #repository_impl
    }
    .into()
}
