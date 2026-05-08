use crate::TableData;
use corrosion_orm_core::schema::relation::RelationType;
use quote::quote;
use syn::Type;

/// Extracts the element type `T` when given a `Vec<T>` type path.
///
/// Returns `None` if the provided type is not a `Vec` path or the `Vec` has no first generic type argument.
///
/// # Examples
///
/// ```ignore
/// use syn::Type;
///
/// let ty: Type = syn::parse_str("Vec<i32>").unwrap();
/// let inner = extract_vec_inner_type(&ty).expect("should extract inner type");
/// if let Type::Path(tp) = inner {
///     assert_eq!(tp.path.segments.last().unwrap().ident, "i32");
/// } else {
///     panic!("expected a Type::Path for the inner type");
/// }
/// ```
fn extract_vec_inner_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
        && segment.ident == "Vec"
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
    {
        return Some(inner_ty);
    }
    None
}

/// Extracts the identifier of the last path segment when `ty` is a path type.
///
/// # Returns
/// `Some(Ident)` if `ty` is a `Type::Path` and the last path segment has an identifier, `None` otherwise.
///
/// # Examples
///
/// ```ignore
/// use syn::{Type, parse_quote};
/// let ty: Type = parse_quote!(std::collections::HashMap<String, i32>);
/// let ident = extract_type_ident(&ty).unwrap();
/// assert_eq!(ident.to_string(), "HashMap");
/// ```
fn extract_type_ident(ty: &Type) -> Option<syn::Ident> {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return Some(segment.ident.clone());
    }
    None
}

/// Generates the repository implementation TokenStream for the given table metadata.
///
/// The produced tokens implement instance helpers (value extraction, id accessors, relation loading)
/// and the `Repo` trait for the entity, including relation-aware cascaded save/delete logic,
/// queries (`save`, `get_all`, `get_by_id`, `delete`) and a `find` helper. Relation handling
/// includes generating code to load associated entities, cascade saves before/after persisting,
/// and cascade deletes where appropriate.
///
/// # Examples
///
/// ```ignore
/// // Given a `TableData` describing an entity, generate the repository implementation tokens:
/// let tokens = generate_repository(&table_data);
/// // Tokens can be inspected or emitted by a procedural macro.
/// assert!(!tokens.to_string().is_empty());
/// ```
pub(crate) fn generate_repository(table: &TableData) -> proc_macro2::TokenStream {
    let ident = &table.ident;
    let primary_key_ty = &table.primary_key.ty;
    let mod_name = syn::Ident::new(
        &table.ident.to_string().to_lowercase(),
        proc_macro2::Span::call_site(),
    );

    let pk_ident = &table.primary_key.iden;
    let field_idents: Vec<_> = table.fields.iter().map(|f| &f.iden).collect();
    let pk_column_variant =
        syn::Ident::new(&table.primary_key.name, proc_macro2::Span::call_site());

    let mut relation_values_push = Vec::new();
    let mut load_relations_impl = Vec::new();
    let mut cascade_save_before_impl = Vec::new();
    let mut cascade_save_after_impl = Vec::new();
    let mut struct_update_impl = Vec::new();

    let mut cascade_delete_before_impl = Vec::new();
    let mut cascade_delete_after_impl = Vec::new();

    for rel in &table.relations {
        let rel_ident = &rel.ident;
        let rel_ty = &rel.ty;
        let fk_name_str = &rel.foreign_key;
        let fk_column = syn::Ident::new(fk_name_str, proc_macro2::Span::call_site());

        match rel.relation_type {
            RelationType::HasOne | RelationType::BelongsTo => {
                relation_values_push.push(quote! {
                    values.push(corrosion_orm_core::query::query_type::Value::from(self.#rel_ident.get_id()));
                });

                cascade_save_before_impl.push(quote! {
                    let #rel_ident = self.#rel_ident.save(db).await?;
                });

                struct_update_impl.push(quote! {
                    entity.#rel_ident = #rel_ident;
                });

                load_relations_impl.push(quote! {
                    if let Some(loaded) = <#rel_ty as corrosion_orm_core::model::repository::Repo<Db>>::get_by_id(self.#rel_ident.get_id(), db).await? {
                        self.#rel_ident = loaded;
                    }
                });

                cascade_delete_after_impl.push(quote! {
                    self.#rel_ident.delete(db).await?;
                });
            }
            RelationType::HasMany | RelationType::BelongsToMany => {
                cascade_save_after_impl.push(quote! {
                    let mut #rel_ident = Vec::new();
                    for item in &self.#rel_ident {
                        let mut __child = (*item).clone();
                        __child.#fk_column = entity.get_id();
                        #rel_ident.push(__child.save(db).await?);
                    }
                });

                struct_update_impl.push(quote! {
                    entity.#rel_ident = #rel_ident;
                });

                let inner_ty = extract_vec_inner_type(rel_ty).unwrap_or(rel_ty);
                let inner_ident = extract_type_ident(inner_ty)
                    .unwrap_or_else(|| syn::Ident::new("Unknown", proc_macro2::Span::call_site()));
                let inner_mod_name = syn::Ident::new(
                    &inner_ident.to_string().to_lowercase(),
                    proc_macro2::Span::call_site(),
                );

                load_relations_impl.push(quote! {
                    let mut __children = <#inner_ty as corrosion_orm_core::model::repository::Repo<Db>>::find()
                        .filter(#inner_mod_name::COLUMN.#fk_column.eq(self.get_id()))
                        .all(db)
                        .await?;
                    for __child in &mut __children {
                        __child.load_relations(db).await?;
                    }
                    self.#rel_ident = __children;
                });

                cascade_delete_before_impl.push(quote! {
                    let __children = <#inner_ty as corrosion_orm_core::model::repository::Repo<Db>>::find()
                        .filter(#inner_mod_name::COLUMN.#fk_column.eq(entity_pk.clone()))
                        .all(db)
                    .await?;
                    for item in __children {
                        item.delete(db).await?;
                    }
                });
            }
        }
    }

    quote! {
        impl #ident {
            fn get_values_from_self(&self) -> Vec<corrosion_orm_core::query::query_type::Value> {
                let mut values = Vec::new();
                values.push(corrosion_orm_core::query::query_type::Value::from(self.#pk_ident.clone()));
                #(
                    values.push(corrosion_orm_core::query::query_type::Value::from(self.#field_idents.clone()));
                )*

                #(#relation_values_push)*

                values
            }

            /// Returns the strongly-typed primary key for this entity
            pub fn get_id(&self) -> #primary_key_ty {
                self.#pk_ident.clone()
            }

            /// Sets the primary key on this entity (used by from_row for relation loading)
            pub fn set_id(&mut self, value: #primary_key_ty) {
                self.#pk_ident = value;
            }

            pub fn get_primary_key_value(&self) -> corrosion_orm_core::query::query_type::Value {
                corrosion_orm_core::query::query_type::Value::from(self.#pk_ident.clone())
            }

            pub(crate) async fn load_relations<Db: corrosion_orm_core::driver::executor::Executor>(&mut self, db: &mut Db) -> Result<(), corrosion_orm_core::error::CorrosionOrmError> {
                #(#load_relations_impl)*
                Ok(())
            }
        }

        impl<Db: corrosion_orm_core::driver::executor::Executor> corrosion_orm_core::model::repository::Repo<Db> for #ident {
            type PrimaryKey = #primary_key_ty;
            type Column = #mod_name::Column;

            async fn save(&self, db: &mut Db) -> Result<Self, corrosion_orm_core::error::CorrosionOrmError> {
                use corrosion_orm_core::query::to_sql::ToSql;
                use corrosion_orm_core::schema::table::TableSchema;
                use corrosion_orm_core::query::where_clause::WhereClause;
                use corrosion_orm_core::query::query_type::QueryContext;

                let schema = Self::get_schema();

                #(#cascade_save_before_impl)*

                let check_query = corrosion_orm_core::query::select::Select::<#mod_name::Column>::from(&schema)
                    .where_clause(
                        WhereClause::eq(#mod_name::Column::#pk_column_variant, self.#pk_ident.clone()),
                );

                let mut ctx_control = corrosion_orm_core::query::query_type::QueryContext::new();
                check_query.to_sql(&mut ctx_control, db.get_dialect());
                let existing = db.fetch_optional::<Self>(&mut ctx_control).await?;

                let mut ctx = QueryContext::new();

                if existing.is_none() {
                    let mut insert_query = corrosion_orm_core::query::insert::Insert::from(&schema)
                        .values(self.get_values_from_self());
                    insert_query.to_sql(&mut ctx, db.get_dialect());
                    db.execute_query(&mut ctx).await?;
                } else {
                    let mut update_query = corrosion_orm_core::query::update::Update::<#mod_name::Column>::from(&schema)
                        .values(self.get_values_from_self())
                        .where_clause(WhereClause::eq(#mod_name::Column::#pk_column_variant, self.#pk_ident.clone()));
                    update_query.to_sql(&mut ctx, db.get_dialect());
                    db.execute_query(&mut ctx).await?;
                }

                let mut fetch_ctx = QueryContext::new();
                let fetch_query = corrosion_orm_core::query::select::Select::<#mod_name::Column>::from(&schema)
                    .where_clause(WhereClause::eq(#mod_name::Column::#pk_column_variant, self.#pk_ident.clone()));
                fetch_query.to_sql(&mut fetch_ctx, db.get_dialect());

                let mut saved = db.fetch_optional::<Self>(&mut fetch_ctx).await?;
                if let Some(ref mut entity) = saved {
                    #(#cascade_save_after_impl)*

                    #(#struct_update_impl)*
                    entity.load_relations(db).await?;
                }
                saved.ok_or(corrosion_orm_core::driver::error::DriverError::NotFound.into())
            }

            async fn get_all(db: &mut Db) -> Result<Vec<Self>, corrosion_orm_core::error::CorrosionOrmError> {
                use corrosion_orm_core::query::to_sql::ToSql;
                use corrosion_orm_core::schema::table::TableSchema;
                use corrosion_orm_core::query::where_clause::WhereClause;
                use corrosion_orm_core::query::query_type::QueryContext;

                let mut ctx = QueryContext::new();
                let schema = Self::get_schema();
                let query = corrosion_orm_core::query::select::Select::<#mod_name::Column>::from(&schema);
                query.to_sql(&mut ctx, db.get_dialect());
                let mut results = db.fetch_all::<Self>(&mut ctx).await?;

                for result in &mut results {
                    result.load_relations(db).await?;
                }
                Ok(results)
            }

            async fn get_by_id(id: Self::PrimaryKey, db: &mut Db) -> Result<Option<Self>, corrosion_orm_core::error::CorrosionOrmError> {
                use corrosion_orm_core::query::to_sql::ToSql;
                use corrosion_orm_core::schema::table::TableSchema;
                use corrosion_orm_core::query::where_clause::WhereClause;
                use corrosion_orm_core::query::query_type::QueryContext;

                let mut ctx = QueryContext::new();
                let schema = Self::get_schema();
                let query = corrosion_orm_core::query::select::Select::<#mod_name::Column>::from(&schema)
                    .where_clause(WhereClause::eq(#mod_name::Column::#pk_column_variant, id.clone()));
                query.to_sql(&mut ctx, db.get_dialect());
                let result = db.fetch_optional::<Self>(&mut ctx).await?;

                if let Some(mut entity) = result {
                    entity.load_relations(db).await?;
                    Ok(Some(entity))
                } else {
                    Ok(None)
                }
            }

            async fn delete(self, db: &mut Db) -> Result<(), corrosion_orm_core::error::CorrosionOrmError> {
                use corrosion_orm_core::query::to_sql::ToSql;
                use corrosion_orm_core::schema::table::TableSchema;
                use corrosion_orm_core::query::where_clause::WhereClause;
                use corrosion_orm_core::query::query_type::QueryContext;
                use corrosion_orm_core::query::delete::Delete;

                let entity_pk = self.#pk_ident.clone();


                #(#cascade_delete_before_impl)*

                let mut ctx = QueryContext::new();
                let schema = Self::get_schema();
                let delete_query = Delete::<#mod_name::Column>::from(&schema)
                    .where_clause(WhereClause::eq(#mod_name::Column::#pk_column_variant, entity_pk));
                delete_query.to_sql(&mut ctx, db.get_dialect());
                db.execute_query(&mut ctx).await?;


                #(#cascade_delete_after_impl)*

                Ok(())
            }

            fn find<'query>() -> corrosion_orm_core::model::Finder<'query, Self, Db, Self::Column> {
                use corrosion_orm_core::query::select::Select;
                use corrosion_orm_core::schema::table::TableSchema;
                let schema = Self::get_schema();
                let select_query = Select::<#mod_name::Column>::from(schema);
                corrosion_orm_core::model::Finder::new(select_query)
            }
        }
    }
}
