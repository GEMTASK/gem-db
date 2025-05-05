#![allow(dead_code)]
#![allow(unused_variables)]

mod test;

#[derive(Debug)]
enum FieldType {
    Int32,
    Int64,
    String,
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
    fields: Vec<Field>,
    records: Vec<u8>,
    storage: Vec<u8>,
    storage_offset: i32,
}

struct Storage {
    records: Vec<u8>,
}

impl Table {
    pub fn new(name: &str, fields: Vec<Field>) -> Table {
        Self {
            name: name.to_string(),
            fields,
            records: vec![0u8; 24],
            storage: vec![0u8; 16],
            storage_offset: 0,
        }
    }

    pub fn set_field(&mut self, field_index: usize, value: i32) {
        let ptr: *mut u8 = self.records.as_mut_ptr();

        unsafe {
            let offset = *ptr.add(field_index * 4) as usize;

            *(ptr.add(offset) as *mut i32) = value;
        }
    }

    pub fn get_field(&mut self, field_index: usize) -> i32 {
        let ptr: *mut u8 = self.records.as_mut_ptr();

        unsafe {
            let offset = *ptr.add(field_index * 4) as usize;

            return *(ptr.add(offset) as *mut i32);
        }
    }
}

fn main() {
    let fields = vec![
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
    ];

    let mut table = Table::new("items", fields);

    let records_ptr: *mut u8 = table.records.as_mut_ptr();
    let storage_ptr: *mut u8 = table.storage.as_mut_ptr();
    let mut offsets: Vec<i16> = vec![];
    let mut offset: usize = table.fields.len() * 2;

    for (i, field) in table.fields.iter().enumerate() {
        unsafe {
            match field.kind {
                FieldType::Int32 => {
                    offset += (4 - offset % 4) % 4;

                    offsets.push(offset as i16);

                    *(records_ptr.add(offset) as *mut i32) = 255;

                    offset += 4;
                }
                FieldType::Int64 => {
                    offset += (8 - offset % 8) % 8;

                    offsets.push(offset as i16);

                    *(records_ptr.add(offset) as *mut i64) = 65535;

                    offset += 8;
                }
                FieldType::String => {
                    offset += (4 - offset % 4) % 4;

                    let value = "À";

                    offsets.push(offset as i16);

                    *(records_ptr.add(offset) as *mut i32) = 0;
                    *(storage_ptr.add(0) as *mut i16) = value.len() as i16;
                    std::ptr::copy_nonoverlapping(value.as_ptr(), storage_ptr.add(2), value.len());

                    offset += 2 + value.len();
                }
            }
        }
    }

    offset = 0;

    for (i, field) in table.fields.iter().enumerate() {
        unsafe {
            *(records_ptr.add(offset) as *mut i16) = offsets[i];

            offset += 2;
        }
    }

    println!("{:#?}", table);

    table.set_field(0, 100);

    println!("{}", table.get_field(0));
}
