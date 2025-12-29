pub const NATIVE_TYPE_DECIMAL: &str = "decimal";
pub const NATIVE_TYPE_NUMERIC: &str = "numeric";

pub const FK_ACTION_CASCADE: &str = "CASCADE";
pub const FK_ACTION_SET_NULL: &str = "SET NULL";
pub const FK_ACTION_SET_DEFAULT: &str = "SET DEFAULT";
pub const FK_ACTION_RESTRICT: &str = "RESTRICT";

pub const CONSTRAINT_PK: &str = "PRIMARY KEY";
pub const CONSTRAINT_UNIQUE: &str = "UNIQUE";

pub const PRECISION_TYPES: [&'static str; 2] = [NATIVE_TYPE_NUMERIC, NATIVE_TYPE_DECIMAL];
