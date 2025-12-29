use crate::providers::postgresql::constants;
use crate::sqlz::{ForeignKeyAction, GenericType, SqlzRow, SqlzValue};
use postgres::types::Type;

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
        name if name == Type::INT2.name() => GenericType::SmallInt,
        name if name == Type::INT4.name() => GenericType::Integer,
        name if name == Type::INT8.name() => GenericType::BigInt,
        name if name == Type::FLOAT4.name() => GenericType::Float,
        name if name == Type::FLOAT8.name() => GenericType::Double,
        name if name == Type::NUMERIC.name() || name == constants::NATIVE_TYPE_DECIMAL => {
            GenericType::Decimal {
                precision: precision.unwrap_or(0) as usize,
                scale: scale.unwrap_or(0) as usize,
            }
        }
        name if name == Type::CHAR.name() || name == Type::BPCHAR.name() => {
            GenericType::Char(char_len.unwrap_or(0) as usize)
        }
        name if name == Type::VARCHAR.name() => {
            GenericType::VarChar(char_len.unwrap_or(0) as usize)
        }
        name if name == Type::TEXT.name() => GenericType::Text,
        name if name == Type::JSON.name() => GenericType::Json,
        name if name == Type::JSONB.name() => GenericType::Json,
        name if name == Type::BYTEA.name() => GenericType::Blob(0),
        name if name == Type::BOOL.name() => GenericType::Boolean,
        name if name == Type::DATE.name() => GenericType::Date,
        name if name == Type::TIMESTAMP.name() => GenericType::Timestamp,
        name if name == Type::TIMESTAMPTZ.name() => GenericType::TimestampTz,
        _ => GenericType::UserDefined(native_type.to_string()),
    }
}

pub fn map_sqlz_to_native_type(generic_type: &GenericType) -> String {
    match generic_type {
        GenericType::TinyInt => Type::INT2.name().to_string(), // PG doesn't have 1-byte int, use int2
        GenericType::SmallInt => Type::INT2.name().to_string(),
        GenericType::Integer => Type::INT4.name().to_string(),
        GenericType::BigInt => Type::INT8.name().to_string(),
        GenericType::Float => Type::FLOAT4.name().to_string(),
        GenericType::Double => Type::FLOAT8.name().to_string(),
        GenericType::Decimal { .. } => Type::NUMERIC.name().to_string(),
        GenericType::VarChar(_) => Type::VARCHAR.name().to_string(),
        GenericType::Char(_) => Type::BPCHAR.name().to_string(),
        GenericType::Text => Type::TEXT.name().to_string(),
        GenericType::Json => Type::JSON.name().to_string(),
        GenericType::Blob(_) => Type::BYTEA.name().to_string(),
        GenericType::Boolean => Type::BOOL.name().to_string(),
        GenericType::Date => Type::DATE.name().to_string(),
        GenericType::Timestamp => Type::TIMESTAMP.name().to_string(),
        GenericType::TimestampTz => Type::TIMESTAMPTZ.name().to_string(),
        GenericType::UserDefined(_) => "text".to_string(),
    }
}

pub fn to_sqlz_value(index: usize, row: &postgres::Row) -> Option<SqlzValue> {
    let column = &row.columns()[index];
    let pg_type = column.type_();

    match *pg_type {
        Type::INT2 => row.get::<_, Option<i16>>(index).map(SqlzValue::SmallInt),
        Type::INT4 => row.get::<_, Option<i32>>(index).map(SqlzValue::Integer),
        Type::INT8 => row.get::<_, Option<i64>>(index).map(SqlzValue::BigInt),
        Type::FLOAT4 => row.get::<_, Option<f32>>(index).map(SqlzValue::Float),
        Type::FLOAT8 => row.get::<_, Option<f64>>(index).map(SqlzValue::Double),
        Type::NUMERIC => row
            .get::<_, Option<rust_decimal::Decimal>>(index)
            .map(|d| SqlzValue::Decimal(d.to_string())),
        Type::CHAR | Type::BPCHAR => row.get::<_, Option<String>>(index).map(SqlzValue::Char),
        Type::VARCHAR => row.get::<_, Option<String>>(index).map(SqlzValue::VarChar),
        Type::TEXT => row.get::<_, Option<String>>(index).map(SqlzValue::Text),
        Type::JSON | Type::JSONB => row
            .get::<_, Option<serde_json::Value>>(index)
            .map(|v| SqlzValue::Json(v.to_string())),
        Type::BYTEA => row.get::<_, Option<Vec<u8>>>(index).map(SqlzValue::Blob),
        Type::BOOL => row.get::<_, Option<bool>>(index).map(SqlzValue::Bool),
        Type::DATE => row
            .get::<_, Option<chrono::NaiveDate>>(index)
            .map(SqlzValue::Date),
        Type::TIMESTAMP => row
            .get::<_, Option<chrono::NaiveDateTime>>(index)
            .map(SqlzValue::Timestamp),
        Type::TIMESTAMPTZ => row
            .get::<_, Option<chrono::DateTime<chrono::Utc>>>(index)
            .map(SqlzValue::TimestampTz),
        _ => {
            // Fallback for custom types (Enums, etc)
            row.get::<_, Option<String>>(index).map(SqlzValue::Text)
        }
    }
}

pub fn convert_row(row: &postgres::Row) -> SqlzRow {
    let mut columns = Vec::new();
    let mut values = Vec::new();

    for (i, column) in row.columns().iter().enumerate() {
        columns.push(column.name().to_string());

        let val = to_sqlz_value(i, row);
        values.push(val.unwrap_or(SqlzValue::Null));
    }

    SqlzRow { columns, values }
}
