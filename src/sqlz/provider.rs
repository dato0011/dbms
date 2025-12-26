use crate::sqlz::{BatchQueryResult, MigrationPlan, RowReadOptions, SqlzResult, Table};

pub trait Provider<T> {
    fn get_tables(&mut self) -> SqlzResult<Vec<Table>>;
    fn tables_exists(&mut self, tables: &[Table]) -> SqlzResult<Vec<String>>;
    fn generate_schema(&mut self, plan: &MigrationPlan) -> SqlzResult<()>;
    fn migrate_data(&mut self, plan: &MigrationPlan) -> SqlzResult<()>;
    fn create_constraints(&mut self, plan: &MigrationPlan) -> SqlzResult<()>;
    fn read_rows(
        &mut self,
        table: &Table,
        options: RowReadOptions,
    ) -> SqlzResult<Box<BatchQueryResult>>;
}
