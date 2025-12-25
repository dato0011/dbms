use crate::providers::postgresql::constants;
use crate::sqlz::{ForeignKeyAction, GenericType, SqlzValue};

pub fn map_fk_action(action: &str) -> ForeignKeyAction {
    match action {
        constants::FK_ACTION_CASCADE => ForeignKeyAction::Cascade,
        constants::FK_ACTION_SET_NULL => ForeignKeyAction::SetNull,
        constants::FK_ACTION_SET_DEFAULT => ForeignKeyAction::SetDefault,
        constants::FK_ACTION_RESTRICT => ForeignKeyAction::Restrict,
        _ => ForeignKeyAction::NoAction,
    }
}

pub fn map_native_type_to_sqlz(
    _data_type: &str,
    native_type: &str,
    char_len: Option<i32>,
    precision: Option<i32>,
    scale: Option<i32>,
) -> GenericType {
    match native_type {
        // Integers
        constants::NATIVE_TYPE_INT => GenericType::TinyInt,
        constants::NATIVE_TYPE_INT2 => GenericType::SmallInt,
        constants::NATIVE_TYPE_INT4 => GenericType::Integer,
        constants::NATIVE_TYPE_INT8 => GenericType::BigInt,

        // Floats
        constants::NATIVE_TYPE_FLOAT4 => GenericType::Float,
        constants::NATIVE_TYPE_FLOAT8 => GenericType::Double,
        constants::NATIVE_TYPE_NUMERIC | constants::NATIVE_TYPE_DECIMAL => GenericType::Decimal {
            precision: precision.unwrap() as usize,
            scale: scale.unwrap() as usize,
        },

        // Strings/Chars
        constants::NATIVE_TYPE_CHAR => GenericType::Char(char_len.unwrap() as usize),
        constants::NATIVE_TYPE_BPCHAR => GenericType::Char(char_len.unwrap_or(0) as usize), // "Blank-padded char"
        constants::NATIVE_TYPE_VARCHAR => GenericType::VarChar(char_len.unwrap_or(0) as usize),
        constants::NATIVE_TYPE_TEXT => GenericType::Text,

        constants::NATIVE_TYPE_BYTEA => GenericType::Blob(0),

        // Booleans
        constants::NATIVE_TYPE_BOOL => GenericType::Boolean,

        // Date/Time
        constants::NATIVE_TYPE_DATE => GenericType::Date,
        constants::NATIVE_TYPE_TIMESTAMP | constants::NATIVE_TYPE_TIMESTAMPTZ => {
            GenericType::Timestamp
        }

        // Fallback for custom types
        _ => GenericType::UserDefined(native_type.to_string()),
    }
}

// pub fn map_sqlz_to_native_type(generic_type: &GenericType) -> String {
//     match generic_type {
//         GenericType::TinyInt => constants::NATIVE_TYPE_INT.to_string(),
//         GenericType::SmallInt => constants::NATIVE_TYPE_INT2.to_string(),
//         GenericType::Integer => constants::NATIVE_TYPE_INT4.to_string(),
//         GenericType::BigInt => constants::NATIVE_TYPE_INT8.to_string(),
//
//         GenericType::Float => constants::NATIVE_TYPE_FLOAT4.to_string(),
//         GenericType::Double => constants::NATIVE_TYPE_FLOAT8.to_string(),
//         GenericType::Decimal { .. } => constants::NATIVE_TYPE_NUMERIC.to_string(),
//
//         GenericType::VarChar(_) => constants::NATIVE_TYPE_VARCHAR.to_string(),
//         GenericType::Char(_) => constants::NATIVE_TYPE_CHAR.to_string(),
//         GenericType::Text => constants::NATIVE_TYPE_TEXT.to_string(),
//
//         GenericType::Blob(_) => constants::NATIVE_TYPE_BYTEA.to_string(),
//
//         GenericType::Boolean => constants::NATIVE_TYPE_BOOL.to_string(),
//
//         GenericType::Date => constants::NATIVE_TYPE_DATE.to_string(),
//         GenericType::Timestamp => constants::NATIVE_TYPE_TIMESTAMP.to_string(),
//
//         GenericType::UserDefined(_) => "text".to_string(),
//     }
// }

pub fn to_sqlz_value(col_type: &str, index: usize, row: &postgres::Row) -> Option<SqlzValue> {
    match col_type {
        // Integers
        constants::NATIVE_TYPE_INT => row.get::<_, Option<i8>>(index).map(SqlzValue::TinyInt),
        constants::NATIVE_TYPE_INT2 => row.get::<_, Option<i16>>(index).map(SqlzValue::SmallInt),
        constants::NATIVE_TYPE_INT4 => row.get::<_, Option<i32>>(index).map(SqlzValue::Integer),
        constants::NATIVE_TYPE_INT8 => row.get::<_, Option<i64>>(index).map(SqlzValue::BigInt),

        // Floats & Numeric
        constants::NATIVE_TYPE_FLOAT4 => row.get::<_, Option<f32>>(index).map(SqlzValue::Float),
        constants::NATIVE_TYPE_FLOAT8 => row.get::<_, Option<f64>>(index).map(SqlzValue::Double),
        constants::NATIVE_TYPE_NUMERIC | constants::NATIVE_TYPE_DECIMAL => {
            row.get::<_, Option<String>>(index).map(SqlzValue::Decimal)
        }

        // Strings
        constants::NATIVE_TYPE_CHAR | constants::NATIVE_TYPE_BPCHAR => {
            row.get::<_, Option<String>>(index).map(SqlzValue::Char)
        }

        constants::NATIVE_TYPE_VARCHAR => {
            row.get::<_, Option<String>>(index).map(SqlzValue::VarChar)
        }
        constants::NATIVE_TYPE_TEXT => row.get::<_, Option<String>>(index).map(SqlzValue::Text),

        // Booleans
        constants::NATIVE_TYPE_BOOL => row.get::<_, Option<bool>>(index).map(SqlzValue::Bool),

        // Date/Time
        constants::NATIVE_TYPE_DATE => row
            .get::<_, Option<chrono::NaiveDate>>(index)
            .map(SqlzValue::Date),
        constants::NATIVE_TYPE_TIMESTAMP | constants::NATIVE_TYPE_TIMESTAMPTZ => row
            .get::<_, Option<chrono::NaiveDateTime>>(index)
            .map(SqlzValue::Timestamp),

        // Blobs
        constants::NATIVE_TYPE_BYTEA => row.get::<_, Option<Vec<u8>>>(index).map(SqlzValue::Blob),

        _ => row.get::<_, Option<String>>(index).map(SqlzValue::Text),
    }
}
