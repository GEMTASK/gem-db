#![allow(dead_code)]
#![allow(unused_variables)]

mod query;
mod table;
mod test;
mod types;

use std::{cell::RefCell, collections::HashMap, sync::Arc};

use table::{Query, Table};
use types::{Field, FieldType, RelationType, Value};

const QUERY: Query = Query::Eq("id", &Value::Int32(255));

fn main() {
    let mut tables = HashMap::<&str, Arc<RefCell<Table>>>::new();

    let items_table = Arc::new(RefCell::new(Table::new("items")));
    let comments_table = Arc::new(RefCell::new(Table::new("comments")));

    tables.insert("items", items_table.clone());
    tables.insert("comments", comments_table.clone());

    //

    items_table.borrow_mut().add_fields(vec![
        Field::new("id", FieldType::Int32),
        Field::new("title", FieldType::String),
        Field::new("estimate", FieldType::Int64),
        Field::new(
            "comments",
            FieldType::Table {
                key: "item_id".to_string(),
                relation_type: RelationType::Array,
                table: comments_table.clone(),
            },
        ),
    ]);

    comments_table.borrow_mut().add_fields(vec![
        Field::new("id", FieldType::Int32),
        Field::new(
            "item_id",
            FieldType::Relation {
                table: items_table.clone(),
            },
        ),
        Field::new("comment", FieldType::Int32),
    ]);

    //

    items_table.borrow_mut().insert(&[
        Value::Int32(255),
        Value::String("ÀÀ".to_string()),
        Value::Int64(65535),
    ]);

    items_table.borrow_mut().insert(&[
        Value::Int32(10),
        Value::String("AA".to_string()),
        Value::Int64(20),
    ]);

    // println!("{:#?}", (*items_table.borrow_mut()));

    comments_table
        .borrow_mut()
        .insert(&[Value::Int32(100), Value::Int32(255), Value::Int32(200)]);

    {
        let items_table_borrow = items_table.borrow_mut();

        items_table_borrow.print(items_table_borrow.select(None));
    }

    // println!("{:#?}", comments_table);
}
