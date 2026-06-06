use corrosion_orm::SqlDriv;

trait Migration {
    fn name(&self) -> &'static str;
    async fn up<D: SqlDriv>(&self, driver: &mut D) -> anyhow::Result<()>;
    async fn down<D: SqlDriv>(&self, driver: &mut D) -> anyhow::Result<()>;
}
