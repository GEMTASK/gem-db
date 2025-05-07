#![allow(dead_code)]
#![allow(unused_variables)]

mod table;

use std::{cell::RefCell, rc::Rc};

use table::{Column, Table, Type, Value};

fn main() {
    let items_table = Table::new(
        "items",
        vec![
            Column::new("id", Type::Int32),
            Column::new("title", Type::String),
            Column::new("estimate", Type::Int64),
        ],
    );

    let items_table_rc = Rc::new(RefCell::new(items_table));

    let mut comments_table = Table::new(
        "comments",
        vec![
            Column::new("id", Type::Int32),
            Column::new(
                "item_id",
                Type::Relation {
                    table: items_table_rc.clone(),
                },
            ),
            Column::new("comment", Type::Int32),
        ],
    );

    (*items_table_rc.borrow_mut()).insert(&[
        Value::Int32(255),
        Value::String("ÀÀ".to_string()),
        Value::Int64(65535),
    ]);

    (*items_table_rc.borrow_mut()).insert(&[
        Value::Int32(10),
        Value::String("AA".to_string()),
        Value::Int64(20),
    ]);

    (*items_table_rc.borrow_mut()).select();

    comments_table.insert(&[Value::Int32(10), Value::Relation(0), Value::Int32(20)]);

    println!("{:#?}", (*items_table_rc.borrow_mut()));
}
