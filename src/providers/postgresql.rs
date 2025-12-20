use crate::sqlz::{Column, GenericType, Provider, SqlzError, SqlzResult, Table};
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
        let rows = self
            .client
            .query(queries::SELECT_TABLES, &[])
            .map_err(|e| SqlzError::DatabaseError(Box::new(e)))?;

        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            let table_name: String = row.get(0);
            let table = fill_columns(
                &mut self.client,
                Table {
                    name: table_name,
                    columns: Vec::new(),
                },
            )?;
            result.push(table);
        }

        Ok(result)
    }
}

fn fill_columns(client: &mut Client, mut table: Table) -> SqlzResult<Table> {
    let column_rows = client
        .query(queries::SELECT_COLUMNS, &[&table.name])
        .map_err(|e| SqlzError::DatabaseError(Box::new(e)))?;

    for row in column_rows {
        let data_type: String = row.get(1);
        let udt_name: String = row.get(7);
        
        // Retrieve as i32 first to match PostgreSQL's INTEGER type
        let char_len: Option<i32> = row.get(2);
        let numeric_precision: Option<i32> = row.get(3);
        let numeric_scale: Option<i32> = row.get(4);

        let col_type = map_native_type_to_sqlz(
            &data_type,
            &udt_name,
            char_len,
            numeric_precision,
            numeric_scale,
        );
        
        table.columns.push(Column {
            name: row.get(0),
            nullable: row.get::<_, String>(5) == "YES",
            col_type,
            native_type: udt_name,
            is_identity: row.get::<_, String>(8) == "YES",
            // Convert i32 to usize safely for the struct
            max_length: char_len.map(|l| l as usize),
        });
    }

    Ok(table)
}

fn map_native_type_to_sqlz(
    _data_type: &str,
    native_type: &str,
    char_len: Option<i32>,
    precision: Option<i32>,
    scale: Option<i32>,
) -> GenericType {
    match native_type {
        // Integers
        "int2" => GenericType::SmallInt,
        "int4" => GenericType::Integer,
        "int8" => GenericType::BigInt,

        // Floats
        "float4" => GenericType::Float,
        "float8" => GenericType::Double,
        "numeric" => GenericType::Decimal {
            precision: precision.unwrap_or(0) as usize,
            scale: scale.unwrap_or(0) as usize,
        },

        // Strings/Chars
        "varchar" => GenericType::VarChar(char_len.unwrap_or(0) as usize),
        "bpchar" => GenericType::Char(char_len.unwrap_or(0) as usize), // "Blank-padded char"
        "text" | "name" => GenericType::Text,

        "bytea" => GenericType::Blob(0),

        // Booleans
        "bool" => GenericType::Boolean,

        // Date/Time
        "date" => GenericType::Date,
        "timestamp" | "timestamptz" => GenericType::Timestamp,

        // Fallback for custom types (like mpaa_rating in dvdrental)
        _ => GenericType::UserDefined(native_type.to_string()),
    }
}

mod queries {
    pub const SELECT_TABLES: &str = "SELECT table_name
            FROM information_schema.tables
            WHERE table_schema = 'public'
              AND table_type = 'BASE TABLE';";

    pub const SELECT_COLUMNS: &str = "SELECT
            column_name,
            data_type,
            character_maximum_length,
            numeric_precision,
            numeric_scale,
            is_nullable,
            column_default,
            udt_name, 
            is_identity
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = $1
        ORDER BY ordinal_position;";
}
