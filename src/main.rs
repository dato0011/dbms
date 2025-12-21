use std::collections::HashSet;
use providers::postgresql::PostgresqlProvider;
use sqlz::Provider;

pub mod sqlz;
pub mod providers;

fn main() {
    let mut provider = PostgresqlProvider::new().unwrap();
    let mut set = HashSet::new();
    if let Ok(tables) = provider.get_tables() {
        for table in tables {
            //println!("Table: {}", table.name);
            for column in table.columns {
                //println!("  - {}", column.name);
                set.insert(column.native_type.clone());
            }
        }
    } else if let Err(e) = provider.get_tables() {
        eprintln!("Failed to fetch tables: {}", e)
    }

    for typ in set {
        println!("Type: {}", typ);
    }
}
