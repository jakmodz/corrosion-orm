mod generator;
mod model;
mod utils;
mod validation_parser;
use crate::generator::generate_impl;
use crate::model::TableData;
use generator::validation_generate::generate_validations;
use model::parse_model;
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, ItemFn, Pat, parse_macro_input};
use validation_parser::parser::parse_validation;

/// Derive macro that generates ORM boilerplate for database entity.
///
/// Annotate a struct with '#[derive(Model)]' to automatically implement all needed traits
///
/// # Container Attributes
///
/// ### Placed at top of struct
///
/// `#[Table]`
/// Possible attributes => name: String
///
/// `#[Index]`
/// Possible attributes => name: String (optional, auto-generated if omitted),
///                        fields: `Vec<String>` (required),
///                        unique: bool (optional, default = false)
///
/// ### Example
/// ```ignore
/// #[derive(Model)]
/// #[Table(name = "users")]
/// #[Index(name = "idx_email", fields = ["email"], unique = true)]
/// struct User;
/// ```
///
/// ### Placed above field of the struct
///
/// `#[Column]`
///  Possible attributes => name: String,
///                         unique: bool,
///                         nullable: bool,
///                         index: bool (optional, creates single-column index)
///
/// # Example
/// ```ignore
/// #[derive(Model)]
/// struct User {
///     #[Column(name = "nm", unique = true, nullable = false)]
///     name: String
/// }
/// ```
///
/// `#[PrimaryKey]`
/// Possible attributes => generation_strategy: GenerationStrategy
///
/// # Example
/// ```ignore
/// #[derive(Model)]
/// struct User {
///     #[Column(name = "nm", unique = true, nullable = false)]
///     #[PrimaryKey(generation_strategy = {GenerationStrategy::AutoIncrement})]
///     id: i64
/// }
/// ```
///
/// # Full Examples
///
/// ### Basic model with field-level index
/// ```ignore
/// #[derive(Model)]
/// #[Table(name = "users")]
/// struct User {
///     #[Column(name = "id")]
///     #[PrimaryKey(generation_strategy = GenerationStrategy::AutoIncrement)]
///     id: i64,
///     #[Column(name = "email", unique = true, index)]
///     email: String,
///     #[Column(name = "username")]
///     username: String,
/// }
/// ```
///
/// ### Model with table-level composite index
/// ```ignore
/// #[derive(Model)]
/// #[Table(name = "orders")]
/// #[Index(name = "idx_user_created", fields = ["user_id", "created_at"], unique = false)]
/// struct Order {
///     #[Column(name = "id")]
///     #[PrimaryKey(generation_strategy = GenerationStrategy::AutoIncrement)]
///     id: i64,
///     #[Column(name = "user_id")]
///     user_id: i64,
///     #[Column(name = "created_at")]
///     created_at: String,
///     #[Column(name = "amount")]
///     amount: f64,
/// }
/// ```
///
/// ### Model with multiple indexes
/// ```ignore
/// #[derive(Model)]
/// #[Table(name = "products")]
/// #[Index(fields = ["category", "price"])]
/// #[Index(name = "idx_unique_sku", fields = ["sku"], unique = true)]
/// struct Product {
///     #[Column(name = "id")]
///     #[PrimaryKey(generation_strategy = {GenerationStrategy::AutoIncrement})]
///     id: i64,
///     #[Column(name = "sku", index)]
///     sku: String,
///     #[Column(name = "category")]
///     category: String,
///     #[Column(name = "price")]
///     price: f64,
/// }
/// ```
#[proc_macro_derive(Model, attributes(Table, Column, PrimaryKey, Index, HasOne))]
pub fn model_derive(input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);
    let model = match parse_model(&mut ast) {
        Ok(table) => table,
        Err(e) => return e.into_compile_error().into(),
    };

    generate_impl(&model)
}
#[proc_macro_derive(Validate, attributes(NotNull, Size, Pattern, Email))]
pub fn validate_derive(input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);
    let validations = match parse_validation(&mut ast) {
        Ok(validations) => validations,
        Err(e) => return e.into_compile_error().into(),
    };

    generate_validations(&ast.ident, validations).into()
}
#[proc_macro_attribute]
pub fn validate_args(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut func = parse_macro_input!(item as ItemFn);
    let mut validations = Vec::new();
    for arg in func.sig.inputs.iter_mut() {
        if let syn::FnArg::Typed(pat_type) = arg {
            let has_valid = pat_type.attrs.iter().any(|a| a.path().is_ident("valid"));

            if has_valid {
                pat_type.attrs.retain(|a| !a.path().is_ident("valid"));
            }
            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                let arg_name = &pat_ident.ident;
                validations.push(quote! { #arg_name.validate()?;});
            }
        }
    }
    if !validations.is_empty() {
        let original_stms = func.block.stmts;
        func.block = syn::parse_quote!({
            #(#validations)*
            #(#original_stms)*
        });
    }
    quote! {#func}.into()
}
