#![allow(dead_code)]
#![allow(unused_variables)]

mod table;
mod types;

use std::{cell::RefCell, collections::HashMap, sync::Arc};

use table::{Query, Table};
use types::{Field, FieldType, RelationType, Value};

const QUERY: Query = Query::Eq("id", 255);

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

fn main() {
    let items_table = Arc::new(RefCell::new(Table::new("items")));
    let comments_table = Arc::new(RefCell::new(Table::new("comments")));

    let tables = HashMap::<&str, Arc<RefCell<Table>>>::from([
        ("items", items_table.clone()),
        ("comments", comments_table.clone()),
    ]);

    //

    items_table.borrow_mut().add_fields(vec![
        Field::new("id", FieldType::Int32),
        Field::new("title", FieldType::String),
        Field::new("estimate", FieldType::Int64),
        Field::new(
            "comments",
            FieldType::Table {
                key: "itemId".to_string(),
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

    comments_table.borrow_mut().insert(&[
        Value::Int32(100),
        Value::Relation(255),
        Value::Int32(200),
    ]);

    {
        let items_table_borrow = items_table.borrow_mut();

        items_table_borrow.print(items_table_borrow.select(Some(&QUERY)));
    }

    // println!("{:#?}", comments_table);
}
