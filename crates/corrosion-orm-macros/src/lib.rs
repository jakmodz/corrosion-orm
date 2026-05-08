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
///
/// # Relationships
/// Corrosion ORM supports strongly-typed database relationships using three attributes:
/// `#[HasMany]`, `#[BelongsTo]`, and `#[HasOne]`.
///
/// ### The Explicit Foreign Key Rule
/// Because Corrosion ORM provides a compile-time type-safe query builder, any table that
/// contains a foreign key **must explicitly define that physical column as a standard field**.
/// This ensures the generated `COLUMN` enum includes the foreign key for type-safe queries
/// (e.g., `post::COLUMN.teacher_id.eq(1)`).
///
/// ---
///
/// ### `#[HasMany]` (One-to-Many)
/// Defines the "One" side of a One-to-Many relationship. This attribute is placed on a `Vec<T>` field.
/// **Notes:** `HasMany` is a virtual relationship. It generates the Rust code to automatically fetch
/// related records, but it **does not** generate physical columns or constraints on the current table.
/// !!!!! The Model also needs to derive Clone trait
///
/// **Attributes:**
/// * `foreign_key`: The column name in the *target* table that points back to this table.
/// * `table`: The name of the *target* table.
///
/// ### `#[BelongsTo]` (Many-to-One)
/// Defines the "Many" side (the owning side) of a relationship. It indicates that the current
/// table holds the physical foreign key. During DDL generation, this attribute creates a
/// SQL `FOREIGN KEY` constraint linking back to the parent table.
///
/// **Attributes:**
/// * `foreign_key`: The explicit column name in *this* table that holds the parent's ID.
/// * `table`: The name of the *parent* table.
///
/// ### `#[HasOne]` (One-to-One)
/// Similar to `BelongsTo`, but strictly enforces a One-to-One relationship. During DDL generation,
/// it creates both a `FOREIGN KEY` constraint and a `UNIQUE` constraint on the column to guarantee
/// a 1:1 mapping.
///
/// ---
///
/// # Full Relationship Example
/// Demonstrating a 1-to-N relationship between `Teacher` and `Post`.
///
/// ```rust
/// use corrosion_orm_macros::Model;
///
/// // --- The "One" Side ---
/// #[derive(Model, Default)]
/// #[Table(name = "teachers")]
/// pub struct Teacher {
///     #[Column(name = "id")]
///     #[PrimaryKey]
///     pub id: i64,
///
///     #[Column(name = "name")]
///     pub name: String,
///
///     // Virtual relation: Fetches `Vec<Post>` automatically. Generates NO sql columns here.
///     #[HasMany(foreign_key = "teacher_id", table = "posts")]
///     pub posts: Vec<Post>,
/// }
///
/// // --- The "Many" Side ---
/// #[derive(Model, Default)]
/// #[Table(name = "posts")]
/// pub struct Post {
///     #[Column(name = "id")]
///     #[PrimaryKey]
///     pub id: i64,
///
///     // 1. Explicit physical column for the compile-time type-safe query builder
///     #[Column(name = "teacher_id")]
///     pub teacher_id: i64,
///
///     // 2. Relation attribute to generate the SQL FOREIGN KEY constraint and fetch the parent
///     #[BelongsTo(foreign_key = "teacher_id", table = "teachers")]
///     pub teacher: Teacher,
/// }
/// ```
#[proc_macro_derive(
    Model,
    attributes(Table, Column, PrimaryKey, Index, HasOne, HasMany, BelongsTo)
)]
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
