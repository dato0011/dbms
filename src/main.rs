use std::collections::HashSet;
use providers::postgresql::PostgresqlProvider;
use sqlz::Provider;

pub mod sqlz;
pub mod providers;

fn main() {
    let mut provider = PostgresqlProvider::new().unwrap();
    let mut set = HashSet::new();
    match provider.get_tables() {
        Ok(tables) => {
            for table in tables {
                //println!("Table: {}", table.name);
                for column in table.columns {
                    //println!("  - {}", column.name);
                    set.insert(column.native_type);
                }
            }
        }
        Err(e) => eprintln!("Failed to fetch tables: {}", e),
    }
    
    for typ in set {
        println!("Type: {}", typ);
    }
}
