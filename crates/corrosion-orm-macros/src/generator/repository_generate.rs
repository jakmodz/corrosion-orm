use crate::TableData;
use quote::quote;

pub(crate) fn generate_repository(table: &TableData) -> proc_macro2::TokenStream {
    let ident = &table.ident;
    let primary_key_ty = &table.primary_key.ty;

    let mut col_names = vec![table.primary_key.name.as_str()];
    for field in &table.fields {
        col_names.push(field.name.as_str());
    }

    let pk_ident = &table.primary_key.iden;
    let field_idents: Vec<_> = table.fields.iter().map(|f| &f.iden).collect();

    quote! {
        impl #ident{
            fn get_values_from_self(&self) -> Vec<corrosion_orm_core::query::query_type::Value> {
                let mut values = Vec::new();
                values.push(corrosion_orm_core::query::query_type::Value::from(self.#pk_ident.clone()));
                #(
                    values.push(corrosion_orm_core::query::query_type::Value::from(self.#field_idents.clone()));
                )*
                values
            }
        }

        impl<Db: corrosion_orm_core::driver::executor::Executor> corrosion_orm_core::model::repository::Repo<Db> for #ident {
            type PrimaryKey = #primary_key_ty;

            async fn save(&self, db: &mut Db) -> Result<Self, corrosion_orm_core::error::CorrosionOrmError> {
                use corrosion_orm_core::query::to_sql::ToSql;
                use corrosion_orm_core::schema::table::TableSchema;
                use corrosion_orm_core::query::where_clause::WhereClause;
                use corrosion_orm_core::query::query_type::QueryContext;

                let schema = Self::get_schema();

                let check_query = corrosion_orm_core::query::select::Select::from(&schema)
                    .where_clause(
                        WhereClause::eq(&schema.primary_key.name, self.#pk_ident.clone()),
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
                    let mut update_query = corrosion_orm_core::query::update::Update::from(&schema)
                        .values(self.get_values_from_self())
                        .where_clause(WhereClause::eq(&schema.primary_key.name, self.#pk_ident.clone()));
                    update_query.to_sql(&mut ctx, db.get_dialect());
                    db.execute_query(&mut ctx).await?;
                }
                let mut fetch_ctx = QueryContext::new();
                let fetch_query = corrosion_orm_core::query::select::Select::from(&schema)
                    .where_clause(WhereClause::eq(&schema.primary_key.name, self.#pk_ident.clone()));
                fetch_query.to_sql(&mut fetch_ctx, db.get_dialect());
                let saved = db.fetch_optional::<Self>(&mut fetch_ctx).await?;

                saved.ok_or(corrosion_orm_core::driver::error::DriverError::NotFound.into())
            }
            async fn get_all(db: &mut Db) -> Result<Vec<Self>, corrosion_orm_core::error::CorrosionOrmError> {
                use corrosion_orm_core::query::to_sql::ToSql;
                use corrosion_orm_core::schema::table::TableSchema;
                use corrosion_orm_core::query::where_clause::WhereClause;
                use corrosion_orm_core::query::query_type::QueryContext;
                let mut ctx = QueryContext::new();
                let schema = Self::get_schema();
                let query = corrosion_orm_core::query::select::Select::from(&schema);
                query.to_sql(&mut ctx, db.get_dialect());
                let results = db.fetch_all::<Self>(&mut ctx).await?;
                Ok(results)
            }
            async fn get_by_id(id: Self::PrimaryKey, db: &mut Db) -> Result<Self, corrosion_orm_core::error::CorrosionOrmError> {
                use corrosion_orm_core::query::to_sql::ToSql;
                use corrosion_orm_core::schema::table::TableSchema;
                use corrosion_orm_core::query::where_clause::WhereClause;
                use corrosion_orm_core::query::query_type::QueryContext;
                let mut ctx = QueryContext::new();
                let schema = Self::get_schema();
                let query = corrosion_orm_core::query::select::Select::from(&schema)
                    .where_clause(WhereClause::eq(&schema.primary_key.name, id.clone()));
                query.to_sql(&mut ctx, db.get_dialect());
                let result = db.fetch_optional::<Self>(&mut ctx).await?;
                if let Some(result) = result {
                    Ok(result)
                } else {
                    Err(corrosion_orm_core::driver::error::DriverError::NotFound.into())
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
                let mut delete_query = Delete::from(&schema)
                    .where_clause(WhereClause::eq(&schema.primary_key.name, self.#pk_ident.clone()));
                delete_query.to_sql(&mut ctx, db.get_dialect());
                db.execute_query(&mut ctx).await?;
                Ok(())
            }
        }
    }
}
