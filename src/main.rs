#![allow(dead_code)]
#![allow(unused_variables)]

mod table;
use table::{Column, Table, Type, Value};

fn main() {
    let mut items_table = Table::new(
        "items",
        vec![
            Column::new("id", Type::Int32),
            Column::new("title", Type::String),
            Column::new("estimate", Type::Int64),
        ],
    );

    let comments_table = Table::new(
        "comments",
        vec![
            Column::new("id", Type::Int32),
            Column::new(
                "item_id",
                Type::Relation {
                    table: &items_table,
                },
            ),
            Column::new("comment", Type::Int32),
        ],
    );

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

    println!("{:#?}", items_table);
}
