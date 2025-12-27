use super::{helper, queries, introspection};
use crate::sqlz::{BatchQueryResult, ContinueFrom, MigrationPlan, Provider, RowReadOptions, SqlzError, SqlzResult, SqlzRow, Table};
use postgres::types::ToSql;
use postgres::{Client, NoTls};

pub struct PostgresqlProvider {
    client: Client,
}

impl PostgresqlProvider {
    pub fn new() -> SqlzResult<impl Provider> {
        Ok(PostgresqlProvider {
            client: Client::connect(
                "host=localhost user=postgres password=111 dbname=dvdrental",
                NoTls,
            )
            .map_err(|e| SqlzError::ConnectionError(e.to_string()))?,
        })
    }
}

impl Provider for PostgresqlProvider {
    fn get_tables(&mut self) -> SqlzResult<Vec<Table>> {
        introspection::get_all_tables(&mut self.client)
    }

    fn tables_exists(&mut self, tables: &[Table]) -> SqlzResult<Vec<String>> {
        let table_names: Vec<&str> = tables.iter().map(|t| t.name.as_str()).collect();

        let rows = self
            .client
            .query(queries::SELECT_TABLES_EXISTS, &[&table_names])
            .map_err(|e| SqlzError::DatabaseError(Box::new(e)))?;

        Ok(rows.iter().map(|row| row.get("table_name")).collect())
    }

    fn generate_schema(&mut self, plan: &MigrationPlan) -> SqlzResult<()> {
        if !plan.is_valid() {
            return Err(SqlzError::MappingError(format!(
                "Invalid migration plan for table '{}'",
                plan.table.name
            )));
        }

        let sql = queries::build_create_table_sql(plan);

        self.client
            .batch_execute(&sql)
            .map_err(|e| SqlzError::DatabaseError(Box::new(e)))?;

        Ok(())
    }

    fn migrate_data(&mut self, _plan: &MigrationPlan) -> SqlzResult<()> {
        todo!()
    }

    fn create_constraints(&mut self, _plan: &MigrationPlan) -> SqlzResult<()> {
        todo!()
    }

    fn read_rows(
        &mut self,
        table: &Table,
        options: RowReadOptions,
    ) -> SqlzResult<Box<BatchQueryResult>> {
        options.validate()?;
        
        let pk = table.get_pk().unwrap();
        let (sql, key_values) = queries::build_read_rows_sql(table, &options);

        let params: Vec<&(dyn ToSql + Sync)> = key_values
            .iter()
            .map(|v| v as &(dyn ToSql + Sync))
            .collect();

        let rows = self
            .client
            .query(&sql, &params)
            .map_err(|e| SqlzError::DatabaseError(Box::new(e)))?;

        let rows: Vec<SqlzRow> = rows.iter().map(|row| helper::convert_row(row)).collect();
        let mut where_params = Vec::new();
        let mut result = Box::new(BatchQueryResult { rows, continue_from: None });

        if result.rows.len() == options.batch_size {
            let last_row = result.rows.last().unwrap();
            pk.columns.iter().for_each(|column| {
               where_params.push(last_row.get(column.name.as_str()).unwrap().clone());
            });

            result.continue_from = Some(ContinueFrom {
                primary_key: pk,
                where_params,
            });
        }

        Ok(result)
    }
}
