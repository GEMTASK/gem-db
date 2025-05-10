use std::{cell::RefCell, sync::Arc};

use crate::table::Table;

#[derive(Debug, Clone)]
pub enum FieldType {
    Int32,
    Int64,
    String,
    Table {
        key: String,
        relation_type: RelationType,
        table: Arc<RefCell<Table>>,
    },
    Relation {
        table: Arc<RefCell<Table>>,
    },
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub field_type: FieldType,
}

impl Field {
    pub fn new(name: &str, field_type: FieldType) -> Field {
        Self {
            name: name.to_string(),
            field_type,
        }
    }
}

//

#[derive(Debug, Clone)]
pub enum ColumnType {
    Int32,
    Int64,
    String,
    Relation { table: Arc<RefCell<Table>> },
}

#[derive(Debug, Clone)]
pub enum Value {
    Int32(i32),
    Int64(i64),
    String(String),
    Relation(i32),
    Array(Vec<Vec<Value>>),
}

#[derive(Debug)]
pub struct Column {
    pub name: String,
    pub column_type: ColumnType,
}

impl Column {
    pub fn new(name: &str, column_type: ColumnType) -> Column {
        Self {
            name: name.to_string(),
            column_type,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RelationType {
    Scalar,
    Array,
}

#[derive(Debug)]
pub struct Relation {
    pub name: String,
    pub key: String,
    pub relation_type: RelationType,
    pub table: Arc<RefCell<Table>>,
}
