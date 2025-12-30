use super::{constants, helper, introspection, queries};
use crate::sqlz::{
    BatchQueryResult, ContinueFrom, Provider, RowReadOptions, SqlzError, SqlzResult, SqlzRow, Table,
};
use postgres::types::ToSql;
use postgres::{Client, NoTls};
use std::collections::HashMap;

pub struct PostgresqlProvider {
    client: Client,
}

impl PostgresqlProvider {
    pub fn new(conn_string: &str) -> SqlzResult<impl Provider> {
        let mut client = Client::connect(conn_string, NoTls)
            .map_err(|e| SqlzError::ConnectionError(e.to_string()))?;

        // Ensure the session is always UTC to avoid silent timezone shifts
        // during data extraction and insertion.
        // TODO: Provide as optional argument to the provider constructor
        client
            .execute("SET TIME ZONE 'UTC'", &[])
            .map_err(|e| SqlzError::DatabaseError(Box::new(e)))?;

        Ok(Self { client })
    }
}

impl Provider for PostgresqlProvider {
    fn has_schemas_support(&self) -> bool {
        true
    }

    fn get_tables(&mut self) -> SqlzResult<Vec<Table>> {
        introspection::get_all_tables(&mut self.client)
    }

    fn get_existing_tables(&mut self, tables: &[Table]) -> SqlzResult<Vec<Table>> {
        let rows = self
            .client
            .query(queries::SELECT_TABLES, &[])
            .map_err(|e| SqlzError::DatabaseError(Box::new(e)))?;

        let existing_tables: Vec<Table> = rows
            .iter()
            .filter(|row| {
                let table_schema: &str = row.get(constants::COL_TABLE_SCHEMA);
                let table_name: &str = row.get(constants::COL_TABLE_NAME);

                tables.iter().any(|t| {
                    t.name == table_name
                        && t.schema
                            .as_deref()
                            .unwrap_or(super::constants::SCHEMA_PUBLIC)
                            == table_schema
                })
            })
            .map(|row| {
                introspection::read_table(
                    &mut self.client,
                    row.get(constants::COL_TABLE_SCHEMA),
                    row.get(constants::COL_TABLE_NAME),
                )
            })
            .collect::<SqlzResult<_>>()?;

        Ok(existing_tables)
    }

    fn create_schema(&mut self, schema_name: &str) -> SqlzResult<()> {
        self.client
            .execute(&format!("CREATE SCHEMA IF NOT EXISTS {}", schema_name), &[])
            .map_err(|e| SqlzError::DatabaseError(Box::new(e)))?;
        Ok(())
    }

    fn create_table(&mut self, table: &Table) -> SqlzResult<()> {
        let mut column_map = HashMap::new();
        table.columns.iter().for_each(|c| {
            column_map.insert(c.name.clone(), helper::map_sqlz_to_native_type(&c.col_type));
        });

        let sql = queries::build_create_table_sql(table, column_map);

        self.client
            .execute(&sql, &[])
            .map_err(|e| SqlzError::DatabaseError(Box::new(e)))?;

        Ok(())
    }

    fn create_constraints(&mut self) -> SqlzResult<()> {
        todo!()
    }

    fn read_rows(
        &mut self,
        table: &Table,
        options: &RowReadOptions,
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
        let mut result = Box::new(BatchQueryResult {
            rows,
            continue_from: None,
        });

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

    fn write_rows(&mut self, table: &Table, rows: Vec<SqlzRow>) -> SqlzResult<()> {
        if rows.is_empty() {
            return Ok(());
        }

        let (sql, params) = queries::build_write_rows_sql(table, &rows);

        let params: Vec<&(dyn ToSql + Sync)> =
            params.iter().map(|v| v as &(dyn ToSql + Sync)).collect();

        self.client
            .execute(&sql, &params)
            .map_err(|e| SqlzError::DatabaseError(Box::new(e)))?;

        Ok(())
    }
}
