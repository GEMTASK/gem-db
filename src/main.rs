#![allow(dead_code)]
#![allow(unused_variables)]

mod table;

use table::{Column, Table, Type, Value};

fn main() {
    let columns = vec![
        Column {
            name: "id".to_string(),
            kind: Type::Int32,
        },
        table::Column {
            name: "title".to_string(),
            kind: Type::String,
        },
        Column {
            name: "estimate".to_string(),
            kind: Type::Int64,
        },
    ];

    let mut table = Table::new("items", columns);

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
