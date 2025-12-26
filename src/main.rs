use crate::sqlz::{ContinueFrom, RowReadOptions, SqlzValue};
use providers::postgresql::PostgresqlProvider;
use sqlz::Provider;

pub mod providers;
pub mod sqlz;

fn main() {
    let mut provider = PostgresqlProvider::new().unwrap();
    let tables = provider.get_tables().unwrap();
    let film = tables.iter().find(|t| t.name == "film").unwrap();
    let test = tables.iter().find(|t| t.name == "test").unwrap();
    let mut options = RowReadOptions::default();
    options.continue_from = Some(ContinueFrom::new(test.get_pk().unwrap(), vec![SqlzValue::Integer(1), SqlzValue::Integer(2)]));
    options.batch_size = 100;
    // options.continue_from = Some(ContinueFrom::new(
    //     test.constraints.primary_key.clone(),
    //     vec![1, 2, 3],
    // ));
    provider.read_rows(test, options);
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
