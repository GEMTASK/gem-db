#![allow(dead_code)]
#![allow(unused_variables)]

mod table;
use table::{Column, Table, Type, Value};

fn main() {
    let mut table = Table::new(
        "items",
        vec![
            Column::new("id", Type::Int32),
            Column::new("title", Type::String),
            Column::new("estimate", Type::Int64),
        ],
    );

    table.select();

    table.insert(&[
        Value::Int32(255),
        Value::String("ÀÀ".to_string()),
        Value::Int64(65535),
    ]);

    table.insert(&[
        Value::Int32(10),
        Value::String("AA".to_string()),
        Value::Int64(20),
    ]);

    table.select();

    println!("{:#?}", table);
}
