use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenericType {
    // Fixed-size integer types (hardcoded byte sizes, no parameters)
    TinyInt,  // 1 byte
    SmallInt, // 2 bytes
    Integer,  // 4 bytes
    BigInt,   // 8 bytes

    // Floating-point types (could be fixed or parameterized by precision)
    Float,  // Fixed, e.g., 4 bytes single-precision
    Double, // Fixed, e.g., 8 bytes double-precision

    // String types (parameterized by max length or unlimited)
    VarChar(usize), // Arbitrary max length
    Char(usize),    // Fixed length
    Text,           // Unlimited/variable

    // Binary types (e.g., images)
    Blob(usize),

    // Other common types (expand as needed)
    Boolean,
    Date,
    Timestamp,

    // Add more like Decimal(precision, scale) for numerics with arbitrary precision/scale
    Decimal { precision: usize, scale: usize },
    UserDefined(String)
}

pub struct Column {
    pub name: String,
    pub col_type: GenericType,
    pub native_type: String,
    pub nullable: bool,
    pub is_identity: bool,
    pub max_length: Option<usize>,
}

pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
}

#[derive(Debug)]
pub enum SqlzError {
    ConnectionError(String),
    DatabaseError(Box<dyn Error + Send + Sync>),
}

impl fmt::Display for SqlzError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SqlzError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            SqlzError::DatabaseError(err) => write!(f, "Underlying database error: {}", err),
        }
    }
}

impl Error for SqlzError {}

pub type SqlzResult<T> = Result<T, SqlzError>;

pub trait Provider {
    fn get_tables(&mut self) -> SqlzResult<Vec<Table>>;
}
