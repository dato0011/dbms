#[derive(Debug, Clone, PartialEq)]
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

    // Other common types (expand as needed)
    Boolean,
    Date,
    Timestamp,
    // Add more like Decimal(precision, scale) for numerics with arbitrary precision/scale
    Decimal { precision: usize, scale: usize },
}

pub struct Column {
    pub name: String,
    pub typ: GenericType,
    pub nullable: bool,
    pub is_identity: bool,
}

pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
}

pub trait Provider {
    fn get_tables(&mut self) -> Vec<Table>;
}
