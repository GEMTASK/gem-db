#![allow(dead_code)]
#![allow(unused_variables)]

mod test;

#[derive(Debug)]
enum FieldType {
    Int32,
    Int64,
    String,
    // VarString(i16),
}

#[derive(Debug)]
struct Field {
    name: String,
    kind: FieldType,
}

#[derive(Debug)]
struct Schema {
    fields: Vec<Field>,
}

#[derive(Debug)]
struct Table {
    name: String,
    schema: Schema,
    data: Vec<u8>,
}

impl Table {
    pub fn new(name: &str, schema: Schema) -> Table {
        Self {
            name: name.to_string(),
            schema,
            data: vec![0u8; 16],
        }
    }

    pub fn set_field(&mut self, field_index: usize, value: i32) {
        let ptr: *mut u8 = self.data.as_mut_ptr();
        let mut offset: usize = 0;

        for i in 0..field_index {
            match self.schema.fields[i].kind {
                FieldType::Int32 => offset += 4,
                FieldType::Int64 => offset += 4,
                FieldType::String => offset += 4,
            }
        }
    }
}

fn main() {
    let schema = Schema {
        fields: vec![
            Field {
                name: "id".to_string(),
                kind: FieldType::Int32,
            },
            Field {
                name: "title".to_string(),
                kind: FieldType::String,
            },
            Field {
                name: "estimate".to_string(),
                kind: FieldType::Int64,
            },
        ],
    };

    let mut table = Table::new("items", schema);

    let ptr: *mut u8 = table.data.as_mut_ptr();
    let mut offset: usize = 0;

    /*
        Indexing into field
        Very large string would require 2 byte indexes
        Separate file for variable size strings?

        Field 0 => 0
        Field 1 => 4
        Field 2 => 8

        11 = 4
        1111 = 16
        11111111 = 256
    */

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
