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
/// Annotate a struct with '#[derive(Model)]' to automaticly implement all needed traits
///
/// # Container Attributes
///
/// ### Placed at top of struct
///
/// '#[Table]'
/// possible attributes=> name: String
///
/// ### Example
/// ```
/// #[derive(Model)]
/// #[Table(name = "users")]
/// struct User;
/// ```
///
/// ### Placed above field of the struct
///
/// '#[Column]'
///  possible attributes=> name: String,unique: bool,nullable: bool
///
/// # Example
/// ```
/// #[Model]
/// struct User {
///     #[Column(name= "nm",unique = true,nullable = false)]   
///     name: String
///}
/// ```
///
/// '#[PrimaryKey]'
/// possible attributes=> generation_strategy: GenerationStrategy
/// # Example
/// ```
/// #[Model]
/// struct User {
///     #[Column(name= "nm",unique = true,nullable = false)]   
///     #[PrimaryKey(generation_strategy = GenerationStrategy::AutoIncrement)]
///     id: i64
///}
/// ```
/// # Examples
/// ```
/// #[Model]
/// #[Table(name = "users")]
/// struct User {
///     #[Column(name= "id",unique = true,nullable = false)]   
///     #[PrimaryKey(generation_strategy = GenerationStrategy::AutoIncrement)]
///     id: i64
///     #[Column(name= "nm",unique = true,nullable = false)]   
///     name: String
///}
/// ```
#[proc_macro_derive(Model, attributes(Table, Column, PrimaryKey))]
pub fn model_derive(input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);
    let model = match parse_model(&mut ast) {
        Ok(table) => table,
        Err(e) => return e.into_compile_error().into(),
    };

    generate_impl(&model)
}
