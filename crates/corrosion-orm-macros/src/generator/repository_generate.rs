use crate::TableData;
use quote::quote;

pub(crate) fn generate_repository(table: &TableData) -> proc_macro2::TokenStream {
    let ident = &table.ident;
    let primary_key_ty = &table.primary_key.ty;
    let mod_name = syn::Ident::new(
        &table.ident.to_string().to_lowercase(),
        proc_macro2::Span::call_site(),
    );
    let mut col_names = vec![table.primary_key.name.as_str()];
    for field in &table.fields {
        col_names.push(field.name.as_str());
    }

    let pk_ident = &table.primary_key.iden;
    let field_idents: Vec<_> = table.fields.iter().map(|f| &f.iden).collect();
    let relation_idents: Vec<_> = table.relations.iter().map(|r| &r.ident).collect();
    let relation_types: Vec<_> = table.relations.iter().map(|r| &r.ty).collect();
    let pk_column_variant =
        syn::Ident::new(&table.primary_key.name, proc_macro2::Span::call_site());

    quote! {
        impl #ident{
            fn get_values_from_self(&self) -> Vec<corrosion_orm_core::query::query_type::Value> {
                let mut values = Vec::new();
                values.push(corrosion_orm_core::query::query_type::Value::from(self.#pk_ident.clone()));
                #(
                    values.push(corrosion_orm_core::query::query_type::Value::from(self.#field_idents.clone()));
                )*
                #(
                    values.push(self.#relation_idents.get_primary_key_value());
                )*
                values
            }

            pub fn get_primary_key_value(&self) -> corrosion_orm_core::query::query_type::Value {
                corrosion_orm_core::query::query_type::Value::from(self.#pk_ident.clone())
            }

            async fn load_relations<Db: corrosion_orm_core::driver::executor::Executor>(&mut self, db: &mut Db) -> Result<(), corrosion_orm_core::error::CorrosionOrmError> {
                #(
                    {
                        let rel_pk_val = self.#relation_idents.get_primary_key_value();
                        match rel_pk_val {
                            corrosion_orm_core::query::query_type::Value::Int(rel_id) => {
                                if let Some(loaded) = <#relation_types as corrosion_orm_core::model::repository::Repo<Db>>::get_by_id(rel_id, db).await? {
                                    self.#relation_idents = loaded;
                                }
                            }
                            corrosion_orm_core::query::query_type::Value::Int64(rel_id) => {
                                if let Some(loaded) = <#relation_types as corrosion_orm_core::model::repository::Repo<Db>>::get_by_id(rel_id as i32, db).await? {
                                    self.#relation_idents = loaded;
                                }
                            }
                            _ => {}
                        }
                    }
                )*
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

                #(
                    let mut #relation_idents = self.#relation_idents.save(db).await?;
                )*

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

                // Reattach saved relations
                if let Some(ref mut entity) = saved {
                    #(
                        entity.#relation_idents = #relation_idents;
                    )*
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

                let mut ctx = QueryContext::new();
                let schema = Self::get_schema();
                let mut delete_query = Delete::<#mod_name::Column>::from(&schema)
                    .where_clause(WhereClause::eq(#mod_name::Column::#pk_column_variant, self.#pk_ident.clone()));
                delete_query.to_sql(&mut ctx, db.get_dialect());
                db.execute_query(&mut ctx).await?;
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
