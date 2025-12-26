use std::collections::HashMap;
use crate::sqlz::Table;

pub struct MigrationPlan<'a> {
    pub table: &'a Table,
    pub target_column_type_map: HashMap<String, String>,
}

impl<'a> MigrationPlan<'a> {
    pub fn new(table: &'a Table) -> Self {
        Self {
            table,
            target_column_type_map: HashMap::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        // TODO: Validate that source column type maps to target column types
        self.table
            .columns
            .iter()
            .all(|c| self.target_column_type_map.contains_key(&c.name))
    }
}
