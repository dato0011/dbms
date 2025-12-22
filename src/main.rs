use crate::sqlz::MigrationPlan;
use providers::postgresql::PostgresqlProvider;
use sqlz::Provider;

pub mod providers;
pub mod sqlz;

fn main() {
    let mut provider = PostgresqlProvider::new().unwrap();
    let tables = provider.get_tables().unwrap();
    let film = tables.iter().find(|t| t.name == "film").unwrap();
    let mut plan = MigrationPlan::new(film);
    film.columns.iter().for_each(|c| {
        plan.target_column_type_map.insert(
            c.name.clone(),
            "temp_field".to_string()
            //providers::postgresql::map_sqlz_to_native_type(&c.col_type),
        );
    });
    provider.generate_schema(&plan).unwrap();
}
