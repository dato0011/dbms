use providers::postgresql::PostgresqlProvider;
use crate::sqlz::Provider;

pub mod sqlz;
pub mod providers;

fn main() {
    let mut provider = PostgresqlProvider::new();
    let tables = provider.get_tables();
    for table in tables {
        println!("Table: {}", table.name);
    }
}
