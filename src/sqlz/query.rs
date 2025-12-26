use crate::sqlz::{PrimaryKeyConstraint, SqlzValue};

pub struct SqlzRow {
    pub columns: Vec<String>,
    pub values: Vec<SqlzValue>,
}

pub struct RowReadOptions<'a> {
    pub batch_size: usize,
    pub continue_from: Option<ContinueFrom<'a>>,
}

pub struct BatchQueryResult<'a> {
    rows: Vec<SqlzRow>,
    continue_from: Option<ContinueFrom<'a>>,
}

pub struct ContinueFrom<'a> {
    pub primary_key: &'a PrimaryKeyConstraint,
    pub values: Vec<SqlzValue>,
}

impl SqlzRow {
    pub fn get(&self, column_name: &str) -> Option<&SqlzValue> {
        self.columns
            .iter()
            .position(|c| c == column_name)
            .and_then(|i| self.values.get(i))
    }
}

impl<'a> Default for RowReadOptions<'a> {
    fn default() -> Self {
        Self {
            batch_size: 100,
            continue_from: None,
        }
    }
}

impl<'a> ContinueFrom<'a> {
    pub fn new(pk: &'a PrimaryKeyConstraint, values: Vec<SqlzValue>) -> Self {
        Self {
            primary_key: pk,
            values,
        }
    }
}
