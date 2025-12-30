use crate::sqlz::{ForeignKeyAction, GenericType};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct PrimaryKeyConstraint {
    pub constraint_name: String,
    pub columns: Vec<Rc<Column>>,
}

pub enum ConstraintType {
    PrimaryKey(PrimaryKeyConstraint),
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

#[derive(Debug, Clone)]
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
    pub schema: Option<String>,
    pub columns: Vec<Rc<Column>>,
    pub constraints: Vec<ConstraintType>,
}

impl Table {
    pub fn get_pk(&self) -> Option<PrimaryKeyConstraint> {
        self.constraints.iter().find_map(|c| match c {
            ConstraintType::PrimaryKey(pk) => Some(pk.clone()),
            _ => None,
        })
    }

    pub fn can_migrate_to(&self, other: &Table) -> bool {
        if other.columns.len() < self.columns.len() {
            return false;
        }

        for (source_col, target_col) in self.columns.iter().zip(other.columns.iter()) {
            match (&source_col.col_type, &target_col.col_type) {
                (
                    GenericType::TinyInt,
                    GenericType::TinyInt
                    | GenericType::SmallInt
                    | GenericType::Integer
                    | GenericType::BigInt,
                ) => continue,
                (
                    GenericType::SmallInt,
                    GenericType::SmallInt | GenericType::Integer | GenericType::BigInt,
                ) => continue,
                (GenericType::Integer, GenericType::Integer | GenericType::BigInt) => continue,
                (GenericType::BigInt, GenericType::BigInt) => continue,

                // --- Floating Point Widening ---
                (GenericType::Float, GenericType::Float | GenericType::Double) => continue,
                (GenericType::Double, GenericType::Double) => continue,

                // --- String & Text Rules ---
                // Exception: target is Text, source is Text or UserDefined
                (GenericType::Text | GenericType::UserDefined(_), GenericType::Text) => continue,
                (GenericType::Json, GenericType::Text) => continue,

                // Variable-length string/char checks
                (GenericType::VarChar(s1), GenericType::VarChar(s2)) if s2 >= s1 => continue,
                (GenericType::Char(s1), GenericType::Char(s2)) if s2 >= s1 => continue,

                // Migrating fixed-size strings to Text is always safe
                (GenericType::VarChar(_) | GenericType::Char(_), GenericType::Text) => continue,

                // --- Binary & Bit Rules ---
                (GenericType::Blob(s1), GenericType::Blob(s2)) if s2 >= s1 => continue,
                (GenericType::Bit(s1), GenericType::Bit(s2)) if s2 >= s1 => continue,
                (GenericType::VarBit(s1), GenericType::VarBit(s2)) if s2 >= s1 => continue,

                // --- Decimal Rules ---
                (
                    GenericType::Decimal {
                        precision: p1,
                        scale: s1,
                    },
                    GenericType::Decimal {
                        precision: p2,
                        scale: s2,
                    },
                ) if p2 >= p1 && s2 == s1 => continue,

                // --- Exact Match for others (Boolean, Date, Timestamp, Json, Uuid, etc.) ---
                (t1, t2) if t1 == t2 => continue,

                // If none of the widening or equality rules match, migration is unsafe
                _ => return false,
            }
        }

        true
    }
}
