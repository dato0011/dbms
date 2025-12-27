use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum SqlzError {
    ConnectionError(String),
    DatabaseError(Box<dyn Error + Send + Sync>),
    MappingError(String),
    ValidationError(String),
}

pub type SqlzResult<T> = Result<T, SqlzError>;

impl fmt::Display for SqlzError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SqlzError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            SqlzError::DatabaseError(err) => write!(f, "Underlying database error: {}", err),
            SqlzError::MappingError(msg) => write!(f, "Mapping error: {}", msg),
            SqlzError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for SqlzError {}
