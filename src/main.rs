use crate::sqlz::{RowReadOptions};
use providers::postgresql::PostgresqlProvider;
use sqlz::Provider;

pub mod providers;
pub mod sqlz;

fn main() {
    let mut provider =
        PostgresqlProvider::new("host=localhost user=postgres password=111 dbname=dvdrental")
            .unwrap();
    let mut target_provider =
        PostgresqlProvider::new("host=localhost user=postgres password=111 dbname=target_test")
            .unwrap();
    let tables = provider.get_tables().unwrap();
    let film = tables.iter().find(|t| t.name == "film").unwrap();
    let _test = tables.iter().find(|t| t.name == "test2").unwrap();

    let existing = target_provider.get_existing_tables(&tables).unwrap();
    for table in existing.iter() {
        println!("Existing table: {}", table.name);
        println!("Table {} can migrate to: {}", table.name, film.can_migrate_to(table));
    }
    
    return

    target_provider.create_schema(film.schema.as_ref().unwrap()).unwrap();
    target_provider.create_table(film).unwrap();

    let mut read_options = RowReadOptions::default();
    loop {
        let result = provider.read_rows(film, &read_options).unwrap();
        target_provider.write_rows(film, result.rows).unwrap();

        if result.continue_from.is_none() {
            break;
        }
        read_options.continue_from = result.continue_from;
    }

    // let mut options = RowReadOptions::default();
    // options.continue_from = Some(ContinueFrom::new(test.get_pk().unwrap(), vec![SqlzValue::Integer(1), SqlzValue::Integer(2)]));
    // options.batch_size = 100;
    // provider.read_rows(test, options).unwrap();
    // let mut plan = MigrationPlan::new(film);
    // film.columns.iter().for_each(|c| {
    //     plan.target_column_type_map.insert(
    //         c.name.clone(),
    //         "temp_field".to_string()
    //         //providers::postgresql::map_sqlz_to_native_type(&c.col_type),
    //     );
    // });
    // provider.generate_schema(&plan).unwrap();
}
