use providers::postgresql::PostgresqlProvider;
use sqlz::Provider;
use crate::sqlz::MigrationPlan;

pub mod sqlz;
pub mod providers;

fn main() {
    let mut provider = PostgresqlProvider::new().unwrap();
    let tables = provider.get_tables().unwrap();
    let film = tables.iter().find(|t| t.name == "film").unwrap();
    let mut plan = MigrationPlan::new(film);
    film.columns.iter().for_each(|c| {plan.target_column_type_map.insert(c.name.clone(), String::from("test_col"));});
    provider.generate_schema(&plan).unwrap();
}
