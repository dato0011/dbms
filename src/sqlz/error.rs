use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum SqlzError {
    ConnectionError(String),
    DatabaseError(Box<dyn Error + Send + Sync>),
    MappingError(String),
    ValidationError(String),
    MissingFkTable(String),
    DifferentPkColumns(String, String, String),
}

pub type SqlzResult<T> = Result<T, SqlzError>;

impl fmt::Display for SqlzError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SqlzError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            SqlzError::DatabaseError(err) => write!(f, "Underlying database error: {}", err),
            SqlzError::MappingError(msg) => write!(f, "Mapping error: {}", msg),
            SqlzError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            SqlzError::MissingFkTable(table_name) => {
                write!(f, "Missing foreign key table: {}", table_name)
            }
            SqlzError::DifferentPkColumns(table_name, expected, actual) => {
                write!(
                    f,
                    "Different primary key columns for table {}: expected [{}] but found [{}]",
                    table_name, expected, actual
                )
            }
        }
    }
}

impl Error for SqlzError {}
