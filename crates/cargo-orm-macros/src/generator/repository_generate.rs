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
            fn get_values_from_self(&self) -> Vec<cargo_orm_core::query::query_type::Value> {
                let mut values = Vec::new();
                values.push(cargo_orm_core::query::query_type::Value::from(self.#pk_ident.clone()));
                #(
                    values.push(cargo_orm_core::query::query_type::Value::from(self.#field_idents.clone()));
                )*
                values
            }
        }

        impl<Db: cargo_orm_core::driver::executor::Executor> cargo_orm_core::model::repository::Repo<Db> for #ident {
            type PrimaryKey = #primary_key_ty;

            async fn save(&self, db: &mut Db) -> Result<Self, cargo_orm_core::error::CargoOrmError> {
                use cargo_orm_core::query::to_sql::ToSql;
                use cargo_orm_core::schema::table::TableSchema;
                use cargo_orm_core::query::where_clause::WhereClause;

                let schema = Self::get_schema();

                let is_new_query = cargo_orm_core::query::select::Select::from(&schema)
                    .where_clause(
                        WhereClause::eq(&schema.primary_key.name, self.#pk_ident.clone()),
                );

                let mut ctx_control = cargo_orm_core::query::query_type::QueryContext::new();
                is_new_query.to_sql(&mut ctx_control, db.get_dialect());
                let is_new = db.execute_query(&mut ctx_control).await?;

                let mut ctx = cargo_orm_core::query::query_type::QueryContext::new();
                if is_new == 0 {
                    let mut insert_query = cargo_orm_core::query::insert::Insert::from(&schema)
                        .values(self.get_values_from_self());
                    insert_query.to_sql(&mut ctx, db.get_dialect());
                    db.execute_query(&mut ctx).await?;
                } else {
                    let mut update_query = cargo_orm_core::query::update::Update::from(&schema)
                        .values(self.get_values_from_self())
                        .where_clause(WhereClause::eq(&schema.primary_key.name, self.#pk_ident.clone()));
                    update_query.to_sql(&mut ctx, db.get_dialect());
                    db.execute_query(&mut ctx).await?;
                }
                Ok(self.clone())
            }

            async fn get_all(db: &mut Db) -> Result<Vec<Self>, cargo_orm_core::error::CargoOrmError> {
                todo!()
            }
            async fn get_by_id(id: Self::PrimaryKey, db: &mut Db) -> Result<Self, cargo_orm_core::error::CargoOrmError> {
                todo!()
            }
            async fn delete_by_id(id: Self::PrimaryKey, db: &mut Db) -> Result<(), cargo_orm_core::error::CargoOrmError> {
                todo!()
            }
        }
    }
}
