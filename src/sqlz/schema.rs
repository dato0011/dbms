use std::rc::Rc;
use crate::sqlz::{ForeignKeyAction, GenericType};

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
}
