pub const NATIVE_TYPE_INT: &str = "int";
pub const NATIVE_TYPE_INT2: &str = "int2";
pub const NATIVE_TYPE_INT4: &str = "int4";
pub const NATIVE_TYPE_INT8: &str = "int8";
pub const NATIVE_TYPE_FLOAT4: &str = "float4";
pub const NATIVE_TYPE_FLOAT8: &str = "float8";
pub const NATIVE_TYPE_NUMERIC: &str = "numeric";
pub const NATIVE_TYPE_DECIMAL: &str = "decimal";
pub const NATIVE_TYPE_VARCHAR: &str = "varchar";
pub const NATIVE_TYPE_BPCHAR: &str = "bpchar";
pub const NATIVE_TYPE_TEXT: &str = "text";
pub const NATIVE_TYPE_BYTEA: &str = "bytea";
pub const NATIVE_TYPE_BOOL: &str = "bool";
pub const NATIVE_TYPE_DATE: &str = "date";
pub const NATIVE_TYPE_TIMESTAMP: &str = "timestamp";
pub const NATIVE_TYPE_TIMESTAMPTZ: &str = "timestamptz";

pub const FK_ACTION_CASCADE: &str = "CASCADE";
pub const FK_ACTION_SET_NULL: &str = "SET NULL";
pub const FK_ACTION_SET_DEFAULT: &str = "SET DEFAULT";
pub const FK_ACTION_RESTRICT: &str = "RESTRICT";

pub const CONSTRAINT_PK: &str = "PRIMARY KEY";
pub const CONSTRAINT_UNIQUE: &str = "UNIQUE";

pub const PRECISION_TYPES: [&'static str; 2] = [NATIVE_TYPE_NUMERIC, NATIVE_TYPE_DECIMAL];

