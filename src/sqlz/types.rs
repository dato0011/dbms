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

    VarChar(usize),
    Char(usize),
    Text,
    Json,

    // Binary types (e.g., images)
    Blob(usize),
    Uuid,

    // Other common types (expand as needed)
    Boolean,
    Date,
    Timestamp,
    TimestampTz,
    Time,

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
    Json(String),
    Date(chrono::NaiveDate),          // ISO 8601
    Timestamp(chrono::NaiveDateTime), // ISO 8601
    TimestampTz(chrono::DateTime<chrono::Utc>),
    Time(chrono::NaiveTime),
    Blob(Vec<u8>),
    Uuid(uuid::Uuid),
}

pub enum ForeignKeyAction {
    NoAction,
    Restrict,
    SetNull,
    SetDefault,
    Cascade,
}

impl postgres::types::ToSql for SqlzValue {
    fn to_sql(
        &self,
        ty: &postgres::types::Type,
        out: &mut bytes::BytesMut,
    ) -> Result<postgres::types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        match self {
            SqlzValue::Null => Ok(postgres::types::IsNull::Yes),
            SqlzValue::Bool(v) => v.to_sql(ty, out),
            SqlzValue::TinyInt(v) => (*v as i16).to_sql(ty, out), // Postgres doesn't have a 1-byte int usually, maps to SmallInt
            SqlzValue::SmallInt(v) => v.to_sql(ty, out),
            SqlzValue::Integer(v) => v.to_sql(ty, out),
            SqlzValue::BigInt(v) => v.to_sql(ty, out),
            SqlzValue::Float(v) => v.to_sql(ty, out),
            SqlzValue::Double(v) => v.to_sql(ty, out),
            SqlzValue::Text(v) | SqlzValue::VarChar(v) | SqlzValue::Char(v) => v.to_sql(ty, out),
            SqlzValue::Json(v) => v.to_sql(ty, out),
            SqlzValue::Blob(v) => v.to_sql(ty, out),
            SqlzValue::Uuid(v) => v.to_sql(ty, out),
            SqlzValue::Date(v) => v.to_sql(ty, out),
            SqlzValue::Timestamp(v) => v.to_sql(ty, out),
            SqlzValue::TimestampTz(v) => v.to_sql(ty, out),
            SqlzValue::Time(v) => v.to_sql(ty, out),
            SqlzValue::Decimal(v) => {
                let d: rust_decimal::Decimal = v.parse().map_err(|e| Box::new(e))?;
                d.to_sql(ty, out)
            }
        }
    }

    fn accepts(_: &postgres::types::Type) -> bool {
        true // For simplicity in a generic provider
    }

    fn to_sql_checked(
        &self,
        ty: &postgres::types::Type,
        out: &mut bytes::BytesMut,
    ) -> Result<postgres::types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        self.to_sql(ty, out)
    }
}
