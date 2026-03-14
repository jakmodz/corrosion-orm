use crate::TableData;
use quote::quote;
//TODO: generate full implementation. It is just a stub do not generate warning
pub(crate) fn generate_repository(table: &TableData) -> proc_macro2::TokenStream {
    let ident = &table.ident;
    let primary_key = &table.primary_key.ty;
    quote! {
        use cargo_orm_core::Executor;
        use cargo_orm_core::error::CargoOrmError;
        use cargo_orm_core::model::repository::Repo;
        impl <Db: Executor> Repo<Db> for #ident {
            type PrimaryKey = #primary_key;

            async fn save(&self, db: &Db) -> Result<Self, CargoOrmError>{
                todo!()
            }
            async fn get_all(db: &Db) -> Result<Vec<Self>, CargoOrmError>{
                todo!()
            }
            async fn get_by_id(id: Self::PrimaryKey, db: &Db) -> Result<Self, CargoOrmError>{
                todo!()
            }
            async fn delete_by_id(id: Self::PrimaryKey, db: &Db) -> Result<(), CargoOrmError>{
                todo!()
            }
        }
    }
}
