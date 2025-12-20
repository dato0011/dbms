use crate::sqlz::{Provider, Table};
use postgres::{Client, NoTls};

pub struct PostgresqlProvider {
    client: Client,
}

impl PostgresqlProvider {
    pub fn new() -> impl Provider {
        PostgresqlProvider {
            client: Client::connect("host=localhost user=postgres password=111 dbname=dvdrental", NoTls).unwrap(),
        }
    }
}

impl Provider for PostgresqlProvider {
    fn get_tables(&mut self) -> Vec<Table> {
        //self.client.query(, &[]);
        let rows = self.client.query(queries::SELECT_TABLES, &[]).unwrap();
        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            result.push(Table {
                name: row.get(0),
                columns: Vec::new(),
            });
        }

        result
    }
}

mod queries {
    pub const SELECT_TABLES: &str = "SELECT table_name
            FROM information_schema.tables
            WHERE table_schema = 'public'
              AND table_type = 'BASE TABLE';";
}
