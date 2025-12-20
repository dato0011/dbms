use crate::sqlz::{Provider, Table, Column, GenericType, SqlzResult, SqlzError};
use postgres::{Client, NoTls};

pub struct PostgresqlProvider {
    client: Client,
}

impl PostgresqlProvider {
    pub fn new() -> SqlzResult<impl Provider> {
        Ok(PostgresqlProvider {
            client: Client::connect("host=localhost user=postgres password=111 dbname=dvdrental", NoTls)
                .map_err(|e| SqlzError::ConnectionError(e.to_string()))?,
        })
    }
}

impl Provider for PostgresqlProvider {
    fn get_tables(&mut self) -> SqlzResult<Vec<Table>> {
        let rows = self.client.query(queries::SELECT_TABLES, &[])
            .map_err(|e| SqlzError::DatabaseError(Box::new(e)))?;

        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            let table_name: String = row.get(0);
            let table = fill_columns(&mut self.client, Table {
                name: table_name,
                columns: Vec::new(),
            })?;
            result.push(table);
        }

        Ok(result)
    }
}

fn fill_columns(client: &mut Client, mut table: Table) -> SqlzResult<Table> {
        let column_rows = client.query(queries::SELECT_COLUMNS, &[&table.name])
            .map_err(|e| SqlzError::DatabaseError(Box::new(e)))?;
        
        for row in column_rows {
            // Note: You'll need logic here to map PostgreSQL types (udt_name)
            // to your GenericType enum.
            table.columns.push(Column {
                name: row.get(0),
                col_type: GenericType::Text, // Placeholder for your mapping logic
                nullable: row.get::<_, String>(4) == "YES",
                is_identity: row.get::<_, String>(7) == "YES",
            });
        }
        Ok(table)
    }

mod queries {
    pub const SELECT_TABLES: &str = "SELECT table_name
            FROM information_schema.tables
            WHERE table_schema = 'public'
              AND table_type = 'BASE TABLE';";

    pub const SELECT_COLUMNS: &str = "SELECT
            column_name,
            character_maximum_length,
            numeric_precision,
            numeric_scale,
            is_nullable,
            column_default,
            udt_name,  -- Base type name (useful for enums, domains, etc.)
            is_identity
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = $1
        ORDER BY ordinal_position;";
}
