use crate::TableData;
use crate::utils::extract_inner_type;
use corrosion_orm_core::schema::relation::RelationType;
use quote::quote;
use syn::Type;

/// Extracts the element type `T` when given a `Vec<T>` type path.
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
/// Instead of generating inline query logic, the macro now generates:
/// 1. `get_relations() -> Vec<RelationDescriptor>` — a list of relation descriptors
/// 2. Thin glue methods on the entity (`load_relations`, `save`, `delete`, etc.)
///    that dispatch to the generic helpers in `corrosion_orm_core::model::relation_handler`.
pub(crate) fn generate_repository(table: &TableData) -> proc_macro2::TokenStream {
    let ident = &table.ident;
    let primary_key_ty = &table.primary_key.ty;
    let mod_name = syn::Ident::new(
        &table.ident.to_string().to_lowercase(),
        proc_macro2::Span::call_site(),
    );

    let pk_ident = &table.primary_key.iden;
    let pk_column_variant =
        syn::Ident::new(&table.primary_key.name, proc_macro2::Span::call_site());

    let orm = super::orm_crate_path();
    let cache_ident = syn::Ident::new(
        &format!("{}_CACHE", ident.to_string().to_ascii_uppercase()),
        proc_macro2::Span::call_site(),
    );
    let query_cache_ident = syn::Ident::new(
        &format!("{}_QUERY_CACHE", ident.to_string().to_ascii_uppercase()),
        proc_macro2::Span::call_site(),
    );
    let relation_descriptors: Vec<_> = table
        .relations
        .iter()
        .map(|rel| {
            let field_name = rel.ident.to_string();
            let fk_name = &rel.foreign_key;
            let rel_type = match rel.relation_type {
                RelationType::HasOne => quote! { #orm::schema::relation::RelationType::HasOne },
                RelationType::HasMany => {
                    quote! { #orm::schema::relation::RelationType::HasMany }
                }
                RelationType::BelongsTo => {
                    quote! { #orm::schema::relation::RelationType::BelongsTo }
                }
                RelationType::BelongsToMany => {
                    quote! { #orm::schema::relation::RelationType::BelongsToMany }
                }
            };
            let target_table = match &rel.table {
                Some(t) => quote! { #t },
                None => {
                    let check_type = extract_inner_type(&rel.ty);
                    quote! { <#check_type as #orm::schema::table::TableSchema>::get_table_name() }
                }
            };
            let is_eager = rel.is_eager;
            quote! {
                #orm::model::relation_handler::RelationDescriptor {
                    relation_type: #rel_type,
                    field_name: #field_name,
                    foreign_key: #fk_name,
                    target_table: #target_table,
                    is_eager: #is_eager,
                }
            }
        })
        .collect();

    let mut load_arms = Vec::new();
    let mut cascade_save_before_stmts = Vec::new();
    let mut cascade_save_after_stmts = Vec::new();
    let mut struct_update_stmts = Vec::new();
    let mut cascade_delete_before_stmts = Vec::new();
    let mut cascade_delete_after_stmts = Vec::new();

    for rel in &table.relations {
        let rel_ident = &rel.ident;
        let rel_ty = &rel.ty;
        let fk_name_str = &rel.foreign_key;
        let field_name_str = rel.ident.to_string();

        if !rel.cascade || !rel.is_eager {
            continue;
        }

        match rel.relation_type {
            RelationType::HasOne | RelationType::BelongsTo => {
                cascade_save_before_stmts.push(quote! {
                    let #rel_ident = #orm::model::relation_handler::cascade_save_single(
                        &self.#rel_ident, db
                    ).await?;
                });

                struct_update_stmts.push(quote! {
                    entity.#rel_ident = #rel_ident;
                });

                load_arms.push(quote! {
                    #field_name_str => {
                        if let Some(loaded) = #orm::model::relation_handler::load_single::<Db, #rel_ty>(
                            self.#rel_ident.get_id(), db
                        ).await? {
                            self.#rel_ident = loaded;
                        }
                    }
                });

                cascade_delete_after_stmts.push(quote! {
                    #orm::model::relation_handler::cascade_delete_single(self_mut.#rel_ident, db).await?;
                });
            }
            RelationType::HasMany => {
                let inner_ty = extract_vec_inner_type(rel_ty).unwrap_or(rel_ty);
                let inner_ident = extract_type_ident(inner_ty)
                    .unwrap_or_else(|| syn::Ident::new("Unknown", proc_macro2::Span::call_site()));
                let inner_mod_name = syn::Ident::new(
                    &inner_ident.to_string().to_lowercase(),
                    proc_macro2::Span::call_site(),
                );
                let fk_ident = syn::Ident::new(fk_name_str, proc_macro2::Span::call_site());

                cascade_save_after_stmts.push(quote! {
                    let #rel_ident = #orm::model::relation_handler::cascade_save_many(
                        &self.#rel_ident,
                        &entity.get_id(),
                        |child, pid| { child.#fk_ident = pid.clone(); },
                        db,
                    ).await?;
                });

                struct_update_stmts.push(quote! {
                    entity.#rel_ident = #rel_ident;
                });

                load_arms.push(quote! {
                    #field_name_str => {
                        let mut __children = #orm::model::relation_handler::load_many::<Db, #inner_ty>(
                            #orm::query::query_type::Value::from(self.get_id()),
                            #inner_mod_name::Column::#fk_ident,
                            db,
                        ).await?;
                        for __child in &mut __children {
                            __child.load_relations(db).await?;
                        }
                        self.#rel_ident = __children;
                    }
                });

                cascade_delete_before_stmts.push(quote! {
                    #orm::model::relation_handler::cascade_delete_many::<Db, #inner_ty>(
                        #orm::query::query_type::Value::from(entity_pk.clone()),
                        #inner_mod_name::Column::#fk_ident,
                        db,
                    ).await?;
                });
            }
            _ => {}
        }
    }

    let load_relations_body = if load_arms.is_empty() {
        quote! { Ok(()) }
    } else {
        quote! {
            for desc in <Self as #orm::model::relation_handler::RelationHandler>::get_relations() {
                if !desc.is_eager {
                    continue;
                }
                match desc.field_name {
                    #(#load_arms)*
                    _ => {}
                }
            }
            Ok(())
        }
    };

    quote! {

        impl #ident {
            /// Returns the strongly-typed primary key for this entity
            pub fn get_id(&self) -> #primary_key_ty {
                self.#pk_ident.clone()
            }

            /// Sets the primary key on this entity (used by from_row for relation loading)
            pub fn set_id(&mut self, value: #primary_key_ty) {
                self.#pk_ident = value;
            }

            pub fn get_primary_key_value(&self) -> #orm::query::query_type::Value {
                #orm::query::query_type::Value::from(self.#pk_ident.clone())
            }

            pub(crate) async fn load_relations<Db: #orm::driver::executor::Executor>(&mut self, db: &mut Db) -> Result<(), #orm::error::CorrosionOrmError> {
                #load_relations_body
            }
        }

        impl #orm::model::relation_handler::RelationHandler for #ident {
            fn get_relations() -> Vec<#orm::model::relation_handler::RelationDescriptor> {
                vec![#(#relation_descriptors),*]
            }
        }
        static #cache_ident: std::sync::LazyLock<#orm::model::cache::TieredEntityCache<(usize, #primary_key_ty), #ident>> = std::sync::LazyLock::new(|| {
            #orm::model::cache::TieredEntityCache::new(
                10_000,
                std::time::Duration::from_secs(5 * 60),
                std::time::Duration::from_secs(3 * 60),
            )
        });

        static #query_cache_ident: std::sync::LazyLock<#orm::model::cache::TieredQueryCache<String, Vec<#primary_key_ty>>> = std::sync::LazyLock::new(|| {
            #orm::model::cache::TieredQueryCache::new(10_000)
        });

        impl #orm::model::cache::CacheModel for #ident {
            type PrimaryKey = #primary_key_ty;

            fn cache_id(&self) -> Self::PrimaryKey {
                self.get_id()
            }

            fn entity_cache() -> &'static #orm::model::cache::TieredEntityCache<(usize, Self::PrimaryKey), Self> {
                &#cache_ident
            }

            fn query_cache() -> &'static #orm::model::cache::TieredQueryCache<String, Vec<Self::PrimaryKey>> {
                &#query_cache_ident
            }
        }

        impl<Db: #orm::driver::executor::Executor> #orm::model::repository::Repo<Db> for #ident {
            type PrimaryKey = #primary_key_ty;
            type Column = #mod_name::Column;

            async fn save(&self, db: &mut Db) -> Result<Self, #orm::error::CorrosionOrmError> {
                use #orm::query::to_sql::ToSql;
                use #orm::schema::table::TableSchema;
                use #orm::query::where_clause::WhereClause;
                use #orm::query::query_type::QueryContext;
                use #orm::query::InsertPlanGenerator;
                let __cache_scope = #orm::model::cache::scope_id(db);
                let schema = Self::get_schema();

                #(#cascade_save_before_stmts)*

                let check_query = #orm::query::select::Select::<#mod_name::Column>::from(&schema)
                    .where_clause(
                        WhereClause::eq(#mod_name::Column::#pk_column_variant, self.#pk_ident.clone()),
                );

                let mut ctx_control = #orm::query::query_type::QueryContext::new();
                check_query.to_sql(&mut ctx_control, db.get_dialect());
                let existing = db.fetch_optional::<Self>(&mut ctx_control).await?;

                let mut ctx = QueryContext::new();
                if existing.is_none() {
                    let insert_values = self.get_insert_values_with_db(db).await?;
                    let mut insert_plan = self.generate_insert_plan(insert_values);
                    let insert = insert_plan.to_insert();
                    insert.to_sql(&mut ctx, db.get_dialect());
                    db.execute_query(&mut ctx).await?;
                } else {
                    let update_values = self.get_all_values_with_db(db).await?;
                    let mut update_query = #orm::query::update::Update::<#mod_name::Column>::from(&schema)
                        .values(update_values)
                        .where_clause(WhereClause::eq(#mod_name::Column::#pk_column_variant, self.#pk_ident.clone()));
                    update_query.to_sql(&mut ctx, db.get_dialect());
                    db.execute_query(&mut ctx).await?;
                }
                let mut fetch_ctx = QueryContext::new();
                let last_id = if existing.is_none()
                    && schema.primary_key.generation_type.is_some()
                {
                    db.get_last_id().await?
                } else {
                    #orm::query::query_type::Value::from(self.#pk_ident.clone())
                };
                let fetch_query = #orm::query::select::Select::<#mod_name::Column>::from(&schema)
                    .where_clause(WhereClause::eq(#mod_name::Column::#pk_column_variant, last_id));
                fetch_query.to_sql(&mut fetch_ctx, db.get_dialect());

                let mut saved = db.fetch_optional::<Self>(&mut fetch_ctx).await?;
                if let Some(ref mut entity) = saved {
                    #(#cascade_save_after_stmts)*
                    #(#struct_update_stmts)*
                    entity.load_relations(db).await?;
                    #orm::model::cache::put_entity(__cache_scope, entity).await;
                    #orm::model::cache::invalidate_queries::<Self>();
                }
                saved.ok_or(#orm::driver::error::DriverError::NotFound.into())
            }

            async fn get_all(db: &mut Db) -> Result<Vec<Self>, #orm::error::CorrosionOrmError> {
                use #orm::query::to_sql::ToSql;
                use #orm::schema::table::TableSchema;
                use #orm::query::query_type::QueryContext;

                let __cache_scope = #orm::model::cache::scope_id(db);
                let query_cache_key = "__repo_get_all__".to_string();
                if let Some(ids) = #orm::model::cache::get_query_ids::<Self>(__cache_scope, &query_cache_key) {
                    let mut cached_results = Vec::with_capacity(ids.len());
                    let mut cache_complete = true;

                    for id in ids {
                        if let Some(entity) = #orm::model::cache::get_entity::<Self>(__cache_scope, &id) {
                            cached_results.push(entity);
                        } else {
                            cache_complete = false;
                            break;
                        }
                    }

                    if cache_complete {
                        for entity in &mut cached_results {
                            entity.load_relations(db).await?;
                        }
                        return Ok(cached_results);
                    }
                }

                let mut ctx = QueryContext::new();
                let schema = Self::get_schema();
                let query = #orm::query::select::Select::<#mod_name::Column>::from(&schema);
                query.to_sql(&mut ctx, db.get_dialect());
                let mut results = db.fetch_all::<Self>(&mut ctx).await?;

                let mut ids = Vec::with_capacity(results.len());
                for result in &mut results {
                    result.load_relations(db).await?;
                    ids.push(result.get_id());
                    #orm::model::cache::put_entity(__cache_scope, result).await;
                }
                #orm::model::cache::put_query_ids::<Self>(__cache_scope, query_cache_key, ids).await;
                Ok(results)
            }

            async fn get_by_id(id: Self::PrimaryKey, db: &mut Db) -> Result<Option<Self>, #orm::error::CorrosionOrmError> {
                use #orm::query::to_sql::ToSql;
                use #orm::schema::table::TableSchema;
                use #orm::query::where_clause::WhereClause;
                use #orm::query::query_type::QueryContext;

                let __cache_scope = #orm::model::cache::scope_id(db);
                if let Some(entity) = #orm::model::cache::get_entity::<Self>(__cache_scope, &id) {
                    return Ok(Some(entity));
                }

                let mut ctx = QueryContext::new();
                let schema = Self::get_schema();
                let query = #orm::query::select::Select::<#mod_name::Column>::from(&schema)
                    .where_clause(WhereClause::eq(#mod_name::Column::#pk_column_variant, id.clone()));
                query.to_sql(&mut ctx, db.get_dialect());
                let mut result = db.fetch_optional::<Self>(&mut ctx).await?;

                if let Some(ref mut entity) = result {
                    entity.load_relations(db).await?;
                    #orm::model::cache::put_entity(__cache_scope, entity).await;
                }

                Ok(result)
            }

            async fn delete(self, db: &mut Db) -> Result<(), #orm::error::CorrosionOrmError> {
                use #orm::query::to_sql::ToSql;
                use #orm::schema::table::TableSchema;
                use #orm::query::where_clause::WhereClause;
                use #orm::query::query_type::QueryContext;
                use #orm::query::delete::Delete;

                let __cache_scope = #orm::model::cache::scope_id(db);
                let entity_pk = self.get_id();

                #(#cascade_delete_before_stmts)*

                let mut ctx = QueryContext::new();
                let schema = Self::get_schema();
                let delete_query = Delete::<#mod_name::Column>::from(&schema)
                    .where_clause(WhereClause::eq(#mod_name::Column::#pk_column_variant, entity_pk.clone()));
                delete_query.to_sql(&mut ctx, db.get_dialect());
                db.execute_query(&mut ctx).await?;

                let mut self_mut = self;
                #(#cascade_delete_after_stmts)*
                #orm::model::cache::invalidate_entity::<Self>(__cache_scope, &entity_pk).await;
                #orm::model::cache::invalidate_queries::<Self>();
                Ok(())
            }

            fn find<'query>() -> #orm::model::Finder<'query, Self, Db, Self::Column> {
                use #orm::query::select::Select;
                use #orm::schema::table::TableSchema;
                let schema = Self::get_schema();
                let select_query = Select::<#mod_name::Column>::from(schema);
                #orm::model::Finder::new(select_query)
            }
        }
    }
}
