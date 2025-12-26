use crate::sqlz::{PrimaryKeyConstraint, SqlzValue};

pub struct SqlzRow {
    pub columns: Vec<String>,
    pub values: Vec<SqlzValue>,
}

pub struct RowReadOptions {
    pub batch_size: usize,
    pub continue_from: Option<ContinueFrom>,
}

pub struct BatchQueryResult {
    pub rows: Vec<SqlzRow>,
    pub continue_from: Option<ContinueFrom>
}

pub struct ContinueFrom {
    pub primary_key: PrimaryKeyConstraint,
    pub where_params: Vec<SqlzValue>,
}

impl SqlzRow {
    pub fn get(&self, column_name: &str) -> Option<&SqlzValue> {
        self.columns
            .iter()
            .position(|c| c == column_name)
            .and_then(|i| self.values.get(i))
    }
}

impl Default for RowReadOptions {
    fn default() -> Self {
        Self {
            batch_size: 100,
            continue_from: None,
        }
    }
}

impl ContinueFrom{
    pub fn new(pk: PrimaryKeyConstraint, values: Vec<SqlzValue>) -> Self {
        Self {
            primary_key: pk,
            where_params: values,
        }
    }
}
