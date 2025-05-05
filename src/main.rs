#![allow(dead_code)]
#![allow(unused_variables)]

#[derive(Debug)]

enum FieldType {
    Int32,
    Int64,
    String,
}

#[derive(Debug)]
struct Field<'a> {
    name: &'a str,
    kind: FieldType,
}

#[derive(Debug)]
struct Schema<'a> {
    fields: &'a [Field<'a>],
}

#[derive(Debug)]
struct Table<'a> {
    name: &'a str,
    schema: Schema<'a>,
    data: &'a mut Vec<u8>,
}

fn main() {
    let schema: Schema<'_> = Schema {
        fields: &[
            Field {
                name: "id",
                kind: FieldType::Int32,
            },
            Field {
                name: "title",
                kind: FieldType::String,
            },
            Field {
                name: "estimate",
                kind: FieldType::Int64,
            },
        ],
    };

    let table = Table {
        name: "items",
        schema: schema,
        data: &mut vec![0u8; 16],
    };

    let ptr: *mut u8 = table.data.as_mut_ptr();
    let mut offset: usize = 0;

    for (i, field) in table.schema.fields.iter().enumerate() {
        unsafe {
            match field.kind {
                FieldType::Int32 => {
                    offset += (4 - offset % 4) % 4;

                    *(ptr.add(offset) as *mut i32) = 255;

                    offset += 4;
                }
                FieldType::Int64 => {
                    offset += (8 - offset % 8) % 8;

                    *(ptr.add(offset) as *mut i64) = 65535;

                    offset += 8;
                }
                FieldType::String => {
                    offset += (2 - offset % 2) % 2;

                    let value = "À";

                    *(ptr.add(offset) as *mut i16) = value.len() as i16;
                    std::ptr::copy_nonoverlapping(value.as_ptr(), ptr.add(offset + 2), value.len());

                    offset += 2 + value.len();
                }
            }
        }
    }

    println!("{:#?}", table);
}
