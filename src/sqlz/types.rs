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
    UserDefined(String),
}

#[derive(Debug, Clone)]
pub enum SqlzValue {
    Null,
    Bool(bool),
    TinyInt(i8),
    SmallInt(i16),
    Integer(i32),
    BigInt(i64),
    Float(f32),
    Double(f64),
    Decimal(String), // Store as string to preserve precision across DBs
    Char(String),
    VarChar(String),
    Text(String),
    Bytes(Vec<u8>),
    Date(chrono::NaiveDate),          // ISO 8601
    Timestamp(chrono::NaiveDateTime), // ISO 8601
    Blob(Vec<u8>),
}

pub enum ForeignKeyAction {
    NoAction,
    Restrict,
    SetNull,
    SetDefault,
    Cascade,
}

impl fmt::Display for SqlzValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SqlzValue::Null => write!(f, "NULL"),
            SqlzValue::Bool(b) => write!(f, "{}", b),
            SqlzValue::TinyInt(v) => write!(f, "{}", v),
            SqlzValue::SmallInt(v) => write!(f, "{}", v),
            SqlzValue::Integer(v) => write!(f, "{}", v),
            SqlzValue::BigInt(v) => write!(f, "{}", v),
            SqlzValue::Float(v) => write!(f, "{}", v),
            SqlzValue::Double(v) => write!(f, "{}", v),
            SqlzValue::Decimal(v) => write!(f, "{}", v),
            SqlzValue::Char(v) | SqlzValue::VarChar(v) | SqlzValue::Text(v) => {
                write!(f, "'{}'", v.replace("'", "''")) // Basic SQL escaping
            }
            SqlzValue::Date(v) => write!(f, "'{}'", v),
            SqlzValue::Timestamp(v) => write!(f, "'{}'", v),
            SqlzValue::Bytes(v) | SqlzValue::Blob(v) => {
                write!(f, "X'{:x?}'", v) // Simplified hex representation
            }
        }
    }
}
