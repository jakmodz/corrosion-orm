mod from_row_generate;
mod schema_generate;
use quote::quote;

use crate::{TableData, generator::from_row_generate::generate_from_row};

pub(crate) fn generate_impl(table: &TableData) -> proc_macro::TokenStream {
    let schema_impl = schema_generate::generate_schema_impl(table);
    let from_row_impl = generate_from_row(table);
    quote! {
        #schema_impl
        #from_row_impl
    }
    .into()
}
