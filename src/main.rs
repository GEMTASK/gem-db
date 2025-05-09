#![allow(dead_code)]
#![allow(unused_variables)]

mod table;

use std::{cell::RefCell, sync::Arc};

use table::{Column, ColumnType, Query, RelationType, Table, Value};

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

    //

    items_table.borrow_mut().add_columns(vec![
        Column::new("id", ColumnType::Int32),
        Column::new("title", ColumnType::String),
        Column::new("estimate", ColumnType::Int64),
        Column::new(
            "comments",
            ColumnType::Table {
                key: "itemId".to_string(),
                relation_type: RelationType::Array,
                table: comments_table.clone(),
            },
        ),
    ]);

    comments_table.borrow_mut().add_columns(vec![
        Column::new("id", ColumnType::Int32),
        Column::new(
            "item_id",
            ColumnType::Relation {
                table: items_table.clone(),
            },
        ),
        Column::new("comment", ColumnType::Int32),
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
