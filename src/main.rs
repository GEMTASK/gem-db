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

    let records_ptr: *mut u8 = table.records.as_mut_ptr();
    let storage_ptr: *mut u8 = table.storage.as_mut_ptr();

    let mut offset: usize = 0;

    for (i, field) in table.columns.iter().enumerate() {
        match field.kind {
            Type::Int32 => {
                offset += (4 - offset % 4) % 4;

                unsafe {
                    *(records_ptr.add(offset) as *mut i32) = 255;
                }

                offset += 4;
            }
            Type::Int64 => {
                offset += (8 - offset % 8) % 8;

                unsafe {
                    *(records_ptr.add(offset) as *mut i64) = 65535;
                }

                offset += 8;
            }
            Type::String => {
                offset += (4 - offset % 4) % 4;

                let value = "À";

                unsafe {
                    *(records_ptr.add(offset) as *mut i32) = 0;

                    *(storage_ptr.add(0) as *mut i16) = value.len() as i16;
                    std::ptr::copy_nonoverlapping(value.as_ptr(), storage_ptr.add(2), value.len());
                }

                offset += 2 + value.len();
            }
        }
    }

    table.select();

    table.insert(&[
        Value::Int32(10),
        Value::String("A".to_string()),
        Value::Int64(20),
    ]);

    table.select();

    println!("{:#?}", table);
}
