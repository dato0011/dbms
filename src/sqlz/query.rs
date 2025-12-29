use crate::sqlz::{PrimaryKeyConstraint, SqlzValue};

pub struct SqlzRow {
    pub columns: Vec<String>,
    pub values: Vec<SqlzValue>,
}

pub struct RowReadOptions {
    pub batch_size: usize,
    pub continue_from: Option<ContinueFrom>,
}

impl RowReadOptions {
    pub fn validate(&self) -> crate::sqlz::SqlzResult<()> {
        if self.batch_size == 0 {
            return Err(crate::sqlz::SqlzError::ValidationError(
                "batch_size must be greater than 0".to_string(),
            ));
        }
        if let Some(ContinueFrom {
            primary_key,
            where_params,
        }) = &self.continue_from
        {
            if where_params.len() != primary_key.columns.len() {
                return Err(crate::sqlz::SqlzError::ValidationError(
                    "where_params length must match primary key columns length".to_string(),
                ));
            }
        }
        Ok(())
    }
}

pub struct BatchQueryResult {
    pub rows: Vec<SqlzRow>,
    pub continue_from: Option<ContinueFrom>,
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

impl ContinueFrom {
    pub fn new(pk: PrimaryKeyConstraint, values: Vec<SqlzValue>) -> Self {
        Self {
            primary_key: pk,
            where_params: values,
        }
    }
}
