#![allow(dead_code)]
#![allow(unused_variables)]

mod table;
use table::{Column, Table, Type, Value};

enum Type2<'a> {
    Int32,
    Relation { table: &'a Table2<'a> },
}

pub enum Value2 {
    Int32(i32),
    Relation(i32),
}

pub struct Column2 {
    pub name: String,
    pub kind: Type,
}

struct Table2<'a> {
    pub columns: Vec<Type2<'a>>,
}

impl Table2<'_> {
    pub fn insert(&mut self, values: &[Value2]) {}
}

fn main() {
    let mut xxx = Table2 {
        columns: vec![Type2::Int32],
    };

    let y = Table2 {
        columns: vec![Type2::Relation { table: &xxx }],
    };

    xxx.insert(&[Value2::Int32(5)]);

    //

    let mut items_table = Table::new(
        "items",
        vec![
            Column::new("id", Type::Int32),
            Column::new("title", Type::String),
            Column::new("estimate", Type::Int64),
        ],
    );

    let mut comments_table = Table::new(
        "comments",
        vec![
            Column::new("id", Type::Int32),
            // Column::new("item_id", Type::Relation { table: items_table }),
            Column::new("comment", Type::Int32),
        ],
    );

    items_table.select();

    items_table.insert(&[
        Value::Int32(255),
        Value::String("ÀÀ".to_string()),
        Value::Int64(65535),
    ]);

    items_table.insert(&[
        Value::Int32(10),
        Value::String("AA".to_string()),
        Value::Int64(20),
    ]);

    items_table.select();

    // println!("{:#?}", items_table);
}
