use crate::sqlz::{BatchQueryResult, RowReadOptions, SqlzResult, SqlzRow, Table};

pub trait Provider {
    fn has_schemas_support(&self) -> bool;
    fn get_tables(&mut self) -> SqlzResult<Vec<Table>>;
    fn get_existing_tables(&mut self, tables: &[Table]) -> SqlzResult<Vec<Table>>;
    fn create_schema(&mut self, schema_name: &str) -> SqlzResult<()>;
    fn create_table(&mut self, table: &Table) -> SqlzResult<()>;
    fn create_constraints(&mut self) -> SqlzResult<()>;
    fn read_rows(
        &mut self,
        table: &Table,
        options: &RowReadOptions,
    ) -> SqlzResult<Box<BatchQueryResult>>;
    fn write_rows(&mut self, table: &Table, rows: Vec<SqlzRow>) -> SqlzResult<()>;
}
