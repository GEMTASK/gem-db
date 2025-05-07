#![allow(dead_code)]
#![allow(unused_variables)]

mod table;

use std::{cell::RefCell, rc::Rc};

use table::{Column, Query, Relation, RelationType, Table, Type, Value};

const QUERY: Query = Query::Eq("id", 255);

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

    let comments_table = Table::new(
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

    let comments_table_rc = Rc::new(RefCell::new(comments_table));

    (*items_table_rc.borrow_mut()).add_relations(vec![Relation {
        name: "comments".to_string(),
        key: "item_id".to_string(),
        r#type: RelationType::Array,
        table: comments_table_rc.clone(),
    }]);

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

    // println!("{:#?}", (*items_table_rc.borrow_mut()));

    (*comments_table_rc.borrow_mut()).insert(&[
        Value::Int32(100),
        Value::Relation(255),
        Value::Int32(200),
    ]);

    let items_table_borrow = &(*items_table_rc.borrow_mut());

    items_table_borrow.print(items_table_borrow.select(Some(QUERY)));

    // println!("{:#?}", comments_table);
}
