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
    field_offsets: Vec<usize>,
    row_width: u16,
    storage: Vec<u8>,
    next_storage_offset: i32,
}

struct Storage {
    records: Vec<u8>,
}

impl Table {
    pub fn new(name: &str, fields: Vec<Field>) -> Table {
        let mut field_offsets: Vec<usize> = vec![0; fields.len()];
        let mut offset: usize = 0;

        for (i, field) in fields.iter().enumerate() {
            match field.kind {
                FieldType::Int32 => {
                    offset += (4 - offset % 4) % 4;
                    field_offsets[i] = offset;
                    offset += 4;
                }
                FieldType::Int64 => {
                    offset += (8 - offset % 8) % 8;
                    field_offsets[i] = offset;
                    offset += 8;
                }
                FieldType::String => {
                    offset += (4 - offset % 4) % 4;
                    field_offsets[i] = offset;
                    offset += 4;
                }
            }
        }

        println!("{}", offset);

        Self {
            name: name.to_string(),
            fields,
            records: vec![0u8; 16],
            field_offsets,
            row_width: offset as u16,
            storage: vec![0u8; 8],
            next_storage_offset: 0,
        }
    }

    pub fn set_field<T: Copy>(&mut self, field_index: usize, value: T) {
        let ptr: *mut u8 = self.records.as_mut_ptr();

        unsafe {
            *(ptr.add(self.field_offsets[field_index]) as *mut T) = value;
        }
    }

    pub fn get_field<T: Copy>(&self, field_index: usize) -> T {
        let ptr: *const u8 = self.records.as_ptr();

        unsafe {
            return *(ptr.add(self.field_offsets[field_index]) as *const T);
        }
    }

    pub fn select(&self) {
        let ptr: *const u8 = self.records.as_ptr();

        unsafe {
            for (i, field) in self.fields.iter().enumerate() {
                let offset = self.field_offsets[i] as usize;

                match field.kind {
                    FieldType::Int32 => {
                        println!("> {}", *(ptr.add(offset) as *const i32));
                    }
                    FieldType::Int64 => {
                        println!("> {}", *(ptr.add(offset) as *const i64))
                    }
                    FieldType::String => {
                        println!("> {}", *(ptr.add(offset) as *const i32))
                    }
                }
            }
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
    let mut offset: usize = 0;

    for (i, field) in table.fields.iter().enumerate() {
        unsafe {
            match field.kind {
                FieldType::Int32 => {
                    offset += (4 - offset % 4) % 4;

                    *(records_ptr.add(offset) as *mut i32) = 255;

                    offset += 4;
                }
                FieldType::Int64 => {
                    offset += (8 - offset % 8) % 8;

                    *(records_ptr.add(offset) as *mut i64) = 65535;

                    offset += 8;
                }
                FieldType::String => {
                    offset += (4 - offset % 4) % 4;

                    let value = "À";

                    *(records_ptr.add(offset) as *mut i32) = 0;
                    *(storage_ptr.add(0) as *mut i16) = value.len() as i16;
                    std::ptr::copy_nonoverlapping(value.as_ptr(), storage_ptr.add(2), value.len());

                    offset += 2 + value.len();
                }
            }
        }
    }

    println!("{:#?}", table);

    println!("{}", table.get_field::<i32>(0));
    println!("{}", table.get_field::<i32>(1));
    println!("{}", table.get_field::<i32>(2));

    // table.set_field::<i32>(0, 100);

    table.select();
}
