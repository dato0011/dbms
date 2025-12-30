use super::{constants, helper, queries};
use crate::sqlz::{Column, ConstraintType, PrimaryKeyConstraint, SqlzError, SqlzResult, Table};
use postgres::Client;
use std::collections::HashMap;
use std::rc::Rc;

pub fn get_all_tables(client: &mut Client) -> SqlzResult<Vec<Table>> {
    let rows = client
        .query(queries::SELECT_TABLES, &[])
        .map_err(|e| SqlzError::DatabaseError(Box::new(e)))?;

    let mut tables: Vec<Table> = rows
        .into_iter()
        .map(|row| {
            read_table(
                client,
                row.get(constants::COL_TABLE_SCHEMA),
                row.get(constants::COL_TABLE_NAME),
            )
        })
        .collect::<SqlzResult<_>>()?;

    let column_lookup: HashMap<(String, String, String), Rc<Column>> = tables
        .iter()
        .flat_map(|t| {
            t.columns.iter().map(|c| {
                (
                    (t.schema.clone().unwrap(), t.name.clone(), c.name.clone()),
                    Rc::clone(c),
                )
            })
        })
        .collect();

    for table in tables.iter_mut() {
        fill_constraints(client, table)?;
        fill_foreign_keys(client, table, &column_lookup)?;
    }

    Ok(tables)
}

pub fn read_table(
    client: &mut Client,
    schema_name: String,
    table_name: String,
) -> SqlzResult<Table> {
    let mut table = Table {
        schema: Some(schema_name),
        name: table_name,
        columns: Vec::new(),
        constraints: Vec::new(),
    };

    let rows = client
        .query(
            queries::SELECT_COLUMNS,
            &[&table.name, table.schema.as_ref().unwrap()],
        )
        .map_err(|e| SqlzError::DatabaseError(Box::new(e)))?;

    for row in rows {
        let data_type: String = row.get("data_type");
        let udt_name: String = row.get("udt_name");
        let char_len: Option<i32> = row.get("character_maximum_length");
        let mut numeric_precision: Option<i32> = row.get("numeric_precision");
        let mut numeric_scale: Option<i32> = row.get("numeric_scale");

        let col_type = helper::map_native_type_to_sqlz(
            &data_type,
            &udt_name,
            char_len,
            numeric_precision,
            numeric_scale,
        );

        if !constants::PRECISION_TYPES.contains(&udt_name.as_str()) {
            numeric_precision = None;
            numeric_scale = None;
        }

        table.columns.push(Rc::new(Column {
            name: row.get("column_name"),
            is_nullable: row.get::<_, String>("is_nullable") == "YES",
            col_type,
            native_type: udt_name,
            is_identity: row.get::<_, String>("is_identity") == "YES",
            max_length: char_len,
            numeric_precision,
            numeric_scale,
        }));
    }

    Ok(table)
}

fn fill_constraints(client: &mut Client, table: &mut Table) -> SqlzResult<()> {
    let rows = client
        .query(
            queries::SELECT_CONSTRAINTS,
            &[&table.name, table.schema.as_ref().unwrap()],
        )
        .map_err(|e| SqlzError::DatabaseError(Box::new(e)))?;

    for row in rows {
        let constraint_name: String = row.get("constraint_name");
        let constraint_type: String = row.get("constraint_type");
        let column_name: String = row.get("column_name");

        let column = table
            .columns
            .iter()
            .find(|c| c.name == column_name)
            .cloned();

        if let Some(col) = column {
            match constraint_type.as_str() {
                constants::CONSTRAINT_PK => {
                    if let Some(ConstraintType::PrimaryKey(PrimaryKeyConstraint {
                        columns, ..
                    })) = table
                        .constraints
                        .iter_mut()
                        .find(|c| matches!(c, ConstraintType::PrimaryKey { .. }))
                    {
                        columns.push(col);
                    } else {
                        table
                            .constraints
                            .push(ConstraintType::PrimaryKey(PrimaryKeyConstraint {
                                constraint_name,
                                columns: vec![col],
                            }));
                    }
                }
                constants::CONSTRAINT_UNIQUE => {
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
    Ok(())
}

fn fill_foreign_keys(
    client: &mut Client,
    table: &mut Table,
    column_lookup: &HashMap<(String, String, String), Rc<Column>>,
) -> SqlzResult<()> {
    let rows = client
        .query(
            queries::SELECT_FOREIGN_KEYS,
            &[&table.name, table.schema.as_ref().unwrap()],
        )
        .map_err(|e| SqlzError::DatabaseError(Box::new(e)))?;

    for row in rows {
        let column_name: String = row.get("column_name");
        let foreign_schema: String = row.get("foreign_table_schema");
        let foreign_table: String = row.get("foreign_table_name");
        let foreign_column: String = row.get("foreign_column_name");

        let source = table.columns.iter().find(|c| c.name == column_name);
        let target = column_lookup.get(&(foreign_schema, foreign_table.clone(), foreign_column));

        if let (Some(src_col), Some(ref_col)) = (source, target) {
            table.constraints.push(ConstraintType::ForeignKey {
                constraint_name: row.get("constraint_name"),
                source_column: Rc::clone(src_col),
                referenced_table: foreign_table,
                referenced_column: Rc::clone(ref_col),
                on_update: helper::map_fk_action(&row.get::<_, String>("on_update")),
                on_delete: helper::map_fk_action(&row.get::<_, String>("on_delete")),
            });
        }
    }
    Ok(())
}
