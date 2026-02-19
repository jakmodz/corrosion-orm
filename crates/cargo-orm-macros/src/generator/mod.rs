mod schema_generate;
use quote::quote;

use crate::TableData;

pub(crate) fn generate_impl(table: &TableData)->proc_macro::TokenStream{
    
    let schema_impl = schema_generate::generate_schema_impl(table);
    quote! {
        #schema_impl
    }.into()
}