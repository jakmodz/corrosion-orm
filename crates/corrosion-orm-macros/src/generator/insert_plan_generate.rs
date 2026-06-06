use crate::TableData;
use corrosion_orm_core::schema::relation::RelationType;
use corrosion_orm_core::types::generation_strategy::GenerationType;
use quote::quote;

pub fn generate_insert_plan(table: &TableData) -> proc_macro2::TokenStream {
    let orm = super::orm_crate_path();
    let struct_ident = &table.ident;

    let mut insert_columns = Vec::new();
    let mut insert_values_sync = Vec::new();
    let mut insert_values_async = Vec::new();

    let mut all_columns = Vec::new();
    let mut all_values_sync = Vec::new();
    let mut all_values_async = Vec::new();

    let pk_name = &table.primary_key.name;
    let pk_iden = &table.primary_key.iden;
    let pk_col_expr = quote! { String::from(#pk_name) };
    let pk_val_expr = quote! { #orm::query::query_type::Value::from(self.#pk_iden.clone()) };

    all_columns.push(pk_col_expr.clone());
    all_values_sync.push(pk_val_expr.clone());
    all_values_async.push(quote! { values.push(#pk_val_expr); });

    if !matches!(
        table.primary_key.generation_strategy,
        Some(GenerationType::AutoIncrement)
    ) {
        insert_columns.push(pk_col_expr);
        insert_values_sync.push(pk_val_expr);
        insert_values_async.push(
            quote! { values.push(#orm::query::query_type::Value::from(self.#pk_iden.clone())); },
        );
    }

    for field in &table.fields {
        let field_name = &field.name;
        let field_iden = &field.iden;
        let field_col_expr = quote! { String::from(#field_name) };
        let field_val_expr =
            quote! { #orm::query::query_type::Value::from(self.#field_iden.clone()) };

        all_columns.push(field_col_expr.clone());
        all_values_sync.push(field_val_expr.clone());
        all_values_async.push(quote! { values.push(#field_val_expr); });

        if !matches!(
            field.generation_strategy,
            Some(GenerationType::AutoIncrement)
        ) {
            insert_columns.push(field_col_expr);
            insert_values_sync.push(field_val_expr);
            insert_values_async.push(quote! { values.push(#orm::query::query_type::Value::from(self.#field_iden.clone())); });
        }
    }

    for rel in &table.relations {
        match rel.relation_type {
            RelationType::HasOne | RelationType::BelongsTo => {
                let fk_name = &rel.foreign_key;
                let rel_iden = &rel.ident;
                let fk_col_expr = quote! { String::from(#fk_name) };

                all_columns.push(fk_col_expr.clone());
                insert_columns.push(fk_col_expr.clone());

                if rel.is_eager {
                    let val_expr =
                        quote! { #orm::query::query_type::Value::from(self.#rel_iden.get_id()) };
                    all_values_sync.push(val_expr.clone());
                    insert_values_sync.push(val_expr.clone());

                    let push_expr = quote! { values.push(#val_expr); };
                    all_values_async.push(push_expr.clone());
                    insert_values_async.push(push_expr);
                } else {
                    let sync_val_expr = quote! {
                        self.#rel_iden.get_id_value_sync(|e| #orm::query::query_type::Value::from(e.get_id()))
                            .unwrap_or(#orm::query::query_type::Value::Null)
                    };
                    all_values_sync.push(sync_val_expr.clone());
                    insert_values_sync.push(sync_val_expr);

                    let async_push_expr = quote! {
                        let rel_value = self.#rel_iden
                            .resolve_relation_id_value(
                                db,
                                |e| #orm::query::query_type::Value::from(e.get_id())
                            )
                            .await?
                            .ok_or(#orm::driver::error::DriverError::NotFound)?;
                        values.push(rel_value);
                    };
                    all_values_async.push(async_push_expr.clone());
                    insert_values_async.push(async_push_expr);
                }
            }
            _ => {}
        }
    }

    quote! {
        impl #struct_ident {
            pub fn get_insert_columns() -> Vec<String> {
                vec![#(#insert_columns),*]
            }

            pub fn get_insert_values(&self) -> Vec<#orm::query::query_type::Value> {
                vec![#(#insert_values_sync),*]
            }

            pub async fn get_insert_values_with_db<Db: #orm::driver::executor::Executor>(
                &self,
                db: &mut Db,
            ) -> Result<Vec<#orm::query::query_type::Value>, #orm::error::CorrosionOrmError> {
                let mut values = Vec::new();
                #(#insert_values_async)*
                Ok(values)
            }

            pub fn get_all_columns() -> Vec<String> {
                vec![#(#all_columns),*]
            }

            pub fn get_all_values(&self) -> Vec<#orm::query::query_type::Value> {
                vec![#(#all_values_sync),*]
            }

            pub async fn get_all_values_with_db<Db: #orm::driver::executor::Executor>(
                &self,
                db: &mut Db,
            ) -> Result<Vec<#orm::query::query_type::Value>, #orm::error::CorrosionOrmError> {
                let mut values = Vec::new();
                #(#all_values_async)*
                Ok(values)
            }
        }

        impl #orm::query::insert_plan::InsertPlanGenerator for #struct_ident {
            fn generate_insert_plan(&self, values: Vec<#orm::query::query_type::Value>) -> #orm::query::insert_plan::InsertPlan<'_> {
                use #orm::schema::table::TableSchema;

                #orm::query::insert_plan::InsertPlan {
                    table: Self::get_table_name(),
                    columns: Self::get_insert_columns(),
                    values,
                }
            }
        }
    }
}
