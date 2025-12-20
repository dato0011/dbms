use crate::sqlz::{
    Column, ConstraintType, ForeignKeyAction, GenericType, Provider, SqlzError, SqlzResult, Table,
};
use postgres::{Client, NoTls};
use std::collections::HashMap;
use std::rc::Rc;

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

        // 1. Initialize tables and columns
        let mut tables: Vec<Table> = rows
            .into_iter()
            .map(|row| {
                read_table(
                    &mut self.client,
                    Table {
                        name: row.get(0),
                        columns: Vec::new(),
                        constraints: Vec::new(),
                    },
                )
            })
            .collect::<SqlzResult<_>>()?;

        // 2. Build a flat lookup map for columns to avoid O(N^2) searches and borrowing issues
        // Mapping: (table_name, column_name) -> Rc<Column>
        let column_lookup: std::collections::HashMap<(String, String), Rc<Column>> = tables
            .iter()
            .flat_map(|t| {
                t.columns
                    .iter()
                    .map(|c| ((t.name.clone(), c.name.clone()), Rc::clone(c)))
            })
            .collect();

        // 3. Fill constraints and foreign keys using the lookup map
        for table in tables.iter_mut() {
            fill_constraints(&mut self.client, table);
            fill_foreign_keys(&mut self.client, table, &column_lookup);
        }

        Ok(tables)
    }
}

fn read_table(client: &mut Client, mut table: Table) -> SqlzResult<Table> {
    let column_rows = client
        .query(queries::SELECT_COLUMNS, &[&table.name])
        .map_err(|e| SqlzError::DatabaseError(Box::new(e)))?;

    for row in column_rows {
        let data_type: String = row.get(1);
        let udt_name: String = row.get(7);
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

        table.columns.push(Rc::new(Column {
            name: row.get(0),
            nullable: row.get::<_, String>(5) == "YES",
            col_type,
            native_type: udt_name,
            is_identity: row.get::<_, String>(8) == "YES",
            max_length: char_len.map(|l| l as usize),
        }));
    }

    Ok(table)
}

fn fill_constraints(client: &mut Client, table: &mut Table) {
    if let Ok(rows) = client.query(queries::SELECT_CONSTRAINTS, &[&table.name]) {
        for row in rows {
            let constraint_name: String = row.get(0);
            let constraint_type: String = row.get(1);
            let column_name: String = row.get(2);

            let column = table
                .columns
                .iter()
                .find(|c| c.name == column_name)
                .cloned();

            if let Some(col) = column {
                match constraint_type.as_str() {
                    "PRIMARY KEY" => {
                        // Check if we already have this PK (for multi-column PKs)
                        if let Some(ConstraintType::PrimaryKey { columns, .. }) = table
                            .constraints
                            .iter_mut()
                            .find(|c| matches!(c, ConstraintType::PrimaryKey { .. }))
                        {
                            columns.push(col);
                        } else {
                            table.constraints.push(ConstraintType::PrimaryKey {
                                constraint_name,
                                columns: vec![col],
                            });
                        }
                    }
                    "UNIQUE" => {
                        // Group unique constraints by name
                        if let Some(ConstraintType::Unique { columns, .. }) =
                            table.constraints.iter_mut().find(|c| match c {
                                ConstraintType::Unique {
                                    constraint_name: n, ..
                                } => n == &constraint_name,
                                _ => false,
                            })
                        {
                            columns.push(col);
                        } else {
                            table.constraints.push(ConstraintType::Unique {
                                constraint_name,
                                columns: vec![col],
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

fn fill_foreign_keys(
    client: &mut Client,
    table: &mut Table,
    column_lookup: &HashMap<(String, String), Rc<Column>>,
) {
    let rows = match client.query(queries::SELECT_FOREIGN_KEYS, &[&table.name]) {
        Ok(rows) => rows,
        Err(_) => return,
    };

    for row in rows {
        let column_name: String = row.get(1);
        let foreign_table: String = row.get(3);
        let foreign_column: String = row.get(4);

        let source = table.columns.iter().find(|c| c.name == column_name);
        let target = column_lookup.get(&(foreign_table.clone(), foreign_column));

        if let (Some(src_col), Some(ref_col)) = (source, target) {
            table.constraints.push(ConstraintType::ForeignKey {
                constraint_name: row.get(0),
                source_column: Rc::clone(src_col),
                referenced_table: foreign_table,
                referenced_column: Rc::clone(ref_col),
                on_update: map_fk_action(&row.get::<_, String>(5)),
                on_delete: map_fk_action(&row.get::<_, String>(6)),
            });
        }
    }
}

fn map_fk_action(action: &str) -> ForeignKeyAction {
    match action {
        "CASCADE" => ForeignKeyAction::Cascade,
        "SET NULL" => ForeignKeyAction::SetNull,
        "SET DEFAULT" => ForeignKeyAction::SetDefault,
        "RESTRICT" => ForeignKeyAction::Restrict,
        _ => ForeignKeyAction::NoAction,
    }
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
    pub const SELECT_TABLES: &str = "\
        SELECT table_name
        FROM information_schema.tables
        WHERE table_schema = 'public'
          AND table_type = 'BASE TABLE';";

    pub const SELECT_COLUMNS: &str = "\
        SELECT
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

    pub const SELECT_CONSTRAINTS: &str = "\
        SELECT tc.constraint_name, tc.constraint_type, kcu.column_name
        FROM information_schema.table_constraints tc
        JOIN information_schema.key_column_usage kcu
          ON tc.constraint_name = kcu.constraint_name
        WHERE tc.table_schema = 'public'
          AND tc.table_name = $1
          AND tc.constraint_type IN ('PRIMARY KEY', 'UNIQUE');";

    pub const SELECT_FOREIGN_KEYS: &str = "\
        SELECT
            tc.constraint_name,
            kcu.column_name,
            ccu.table_schema AS foreign_table_schema,
            ccu.table_name AS foreign_table_name,
            ccu.column_name AS foreign_column_name,
            rc.update_rule AS on_update,
            rc.delete_rule AS on_delete
        FROM information_schema.table_constraints tc
        JOIN information_schema.key_column_usage kcu
            ON tc.constraint_name = kcu.constraint_name
            AND tc.table_schema = kcu.table_schema
        JOIN information_schema.referential_constraints rc
            ON tc.constraint_name = rc.constraint_name
            AND tc.table_schema = rc.constraint_schema
        JOIN information_schema.constraint_column_usage ccu
            ON rc.unique_constraint_name = ccu.constraint_name
            AND rc.constraint_schema = ccu.table_schema
        WHERE tc.constraint_type = 'FOREIGN KEY'
          AND tc.table_schema = 'public'
          AND tc.table_name = $1
        ORDER BY tc.constraint_name, kcu.ordinal_position;";
}
