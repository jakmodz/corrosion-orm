mod generator;
mod model;
mod utils;
use crate::generator::generate_impl;
use crate::model::TableData;
use model::parse_model;
use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

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
///                        fields: Vec<String> (required),
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
#[proc_macro_derive(Model, attributes(Table, Column, PrimaryKey, Index))]
pub fn model_derive(input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);
    let model = match parse_model(&mut ast) {
        Ok(table) => table,
        Err(e) => return e.into_compile_error().into(),
    };

    generate_impl(&model)
}
