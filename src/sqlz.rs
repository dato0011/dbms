use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::rc::Rc;

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
    Date(chrono::NaiveDate),      // ISO 8601
    Timestamp(chrono::NaiveDateTime), // ISO 8601
    Blob(Vec<u8>),
}

pub struct SqlzRow {
    pub columns: Vec<String>,
    pub values: Vec<SqlzValue>,
}

pub enum ForeignKeyAction {
    NoAction,
    Restrict,
    SetNull,
    SetDefault,
    Cascade,
}

pub enum ConstraintType {
    PrimaryKey {
        constraint_name: String,
        columns: Vec<Rc<Column>>,
    },
    Unique {
        constraint_name: String,
        columns: Vec<Rc<Column>>,
    },
    ForeignKey {
        constraint_name: String,
        source_column: Rc<Column>,
        referenced_table: String,
        referenced_column: Rc<Column>,
        on_update: ForeignKeyAction,
        on_delete: ForeignKeyAction,
    },
}

pub struct Column {
    pub name: String,
    pub col_type: GenericType,
    pub native_type: String,
    pub is_nullable: bool,
    pub is_identity: bool,
    pub max_length: Option<i32>,
    pub numeric_precision: Option<i32>,
    pub numeric_scale: Option<i32>,
}

pub struct Table {
    pub name: String,
    pub columns: Vec<Rc<Column>>,
    pub constraints: Vec<ConstraintType>,
}

pub struct MigrationPlan<'a> {
    pub table: &'a Table,
    pub target_column_type_map: HashMap<String, String>,
}

#[derive(Debug)]
pub enum SqlzError {
    ConnectionError(String),
    DatabaseError(Box<dyn Error + Send + Sync>),
    MappingError(String),
}

impl fmt::Display for SqlzError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SqlzError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            SqlzError::DatabaseError(err) => write!(f, "Underlying database error: {}", err),
            SqlzError::MappingError(msg) => write!(f, "Mapping error: {}", msg),
        }
    }
}

impl<'a> MigrationPlan<'a> {
    pub fn new(table: &'a Table) -> Self {
        Self {
            table,
            target_column_type_map: HashMap::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        // TODO: Validate that source column type maps to target column types
        self.table.columns.iter().all(|c| self.target_column_type_map.contains_key(&c.name))
    }
}

impl SqlzRow {
    pub fn get(&self, column_name: &str) -> Option<&SqlzValue> {
        self.columns
            .iter()
            .position(|c| c == column_name)
            .and_then(|i| self.values.get(i))
    }
}

impl Error for SqlzError {}

pub type SqlzResult<T> = Result<T, SqlzError>;

pub trait Provider<T> {
    fn get_tables(&mut self) -> SqlzResult<Vec<Table>>;
    fn tables_exists(&mut self, tables: &[Table]) -> SqlzResult<Vec<String>>;
    fn generate_schema(&mut self, plan: &MigrationPlan) -> SqlzResult<()>;
    fn migrate_data(&mut self, plan: &MigrationPlan) -> SqlzResult<()>;
    fn create_constraints(&mut self, plan: &MigrationPlan) -> SqlzResult<()>;
    fn convert_row(&self, row: &T) -> SqlzRow;
}
