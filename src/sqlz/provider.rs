use crate::sqlz::{BatchQueryResult, RowReadOptions, SqlzResult, SqlzRow, Table};

pub trait Provider {
    fn get_tables(&mut self) -> SqlzResult<Vec<Table>>;
    fn tables_exists(&mut self, tables: &[Table]) -> SqlzResult<Vec<String>>;
    fn generate_schema(&mut self, table: &Table) -> SqlzResult<()>;
    fn create_constraints(&mut self) -> SqlzResult<()>;
    fn read_rows(
        &mut self,
        table: &Table,
        options: &RowReadOptions,
    ) -> SqlzResult<Box<BatchQueryResult>>;
    fn write_rows(&mut self, table: &Table, rows: Vec<SqlzRow>) -> SqlzResult<()>;
}
