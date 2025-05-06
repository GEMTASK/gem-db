#![allow(dead_code)]
#![allow(unused_variables)]

mod test;

#[derive(Debug)]
enum Type {
    Int32,
    Int64,
    String,
}

enum Value {
    Int32(i32),
    Int64(i64),
    String(String),
}

#[derive(Debug)]
struct Column {
    name: String,
    kind: Type,
}

#[derive(Debug)]
struct Schema {
    columns: Vec<Column>,
}

#[derive(Debug)]
struct Table {
    name: String,
    columns: Vec<Column>,
    column_offsets: Vec<usize>,
    row_width: u16,
    records: Vec<u8>,
    next_records_offset: usize,
    storage: Vec<u8>,
    next_storage_offset: usize,
}

struct Storage {
    records: Vec<u8>,
}

impl Table {
    pub fn new(name: &str, columns: Vec<Column>) -> Table {
        let mut column_offsets: Vec<usize> = vec![0; columns.len()];
        let mut offset: usize = 0;

        for (i, field) in columns.iter().enumerate() {
            match field.kind {
                Type::Int32 => {
                    offset += (4 - offset % 4) % 4;
                    column_offsets[i] = offset;
                    offset += 4;
                }
                Type::Int64 => {
                    offset += (8 - offset % 8) % 8;
                    column_offsets[i] = offset;
                    offset += 8;
                }
                Type::String => {
                    offset += (4 - offset % 4) % 4;
                    column_offsets[i] = offset;
                    offset += 4;
                }
            }
        }

        Self {
            name: name.to_string(),
            columns,
            column_offsets,
            row_width: offset as u16,
            records: vec![0u8; 32],
            next_records_offset: 16,
            storage: vec![0u8; 8],
            next_storage_offset: 4,
        }
    }

    pub fn set_field<T: Copy>(&mut self, field_index: usize, value: T) {
        let ptr: *mut u8 = self.records.as_mut_ptr();

        unsafe {
            *(ptr.add(self.column_offsets[field_index]) as *mut T) = value;
        }
    }

    pub fn get_field<T: Copy>(&self, field_index: usize) -> T {
        let ptr: *const u8 = self.records.as_ptr();

        unsafe {
            return *(ptr.add(self.column_offsets[field_index]) as *const T);
        }
    }

    pub fn insert(&mut self, values: &[Value]) {
        let records_ptr: *mut u8 = self.records.as_mut_ptr();
        let storage_ptr: *mut u8 = self.storage.as_mut_ptr();

        for (i, field_offset) in self.column_offsets.iter().enumerate() {
            let offset = self.next_records_offset + field_offset;

            match &values[i] {
                Value::Int32(value) => unsafe {
                    *(records_ptr.add(offset) as *mut i32) = *value;
                },
                Value::Int64(value) => unsafe {
                    *(records_ptr.add(offset) as *mut i64) = *value;
                },
                Value::String(value) => unsafe {
                    *(records_ptr.add(offset) as *mut i32) = self.next_storage_offset as i32;

                    *(storage_ptr.add(self.next_storage_offset) as *mut i16) = value.len() as i16;
                    std::ptr::copy_nonoverlapping(
                        value.as_ptr(),
                        storage_ptr.add(self.next_storage_offset + 2),
                        value.len(),
                    );

                    self.next_storage_offset += value.len() + 2;
                },
            }
        }

        self.next_records_offset += self.row_width as usize;
    }

    pub fn select(&self) {
        let records_ptr: *const u8 = self.records.as_ptr();

        for (i, field) in self.columns.iter().enumerate() {
            print!("{:<12}", field.name);
        }

        println!();

        for (i, field) in self.columns.iter().enumerate() {
            print!("{:<12}", "-".repeat(11));
        }

        println!();

        unsafe {
            for j in 0..self.next_records_offset / self.row_width as usize {
                for (i, field) in self.columns.iter().enumerate() {
                    let offset = self.row_width as usize * j + self.column_offsets[i] as usize;

                    match field.kind {
                        Type::Int32 => {
                            print!("{:<12}", *(records_ptr.add(offset) as *const i32));
                        }
                        Type::Int64 => {
                            print!("{:<12}", *(records_ptr.add(offset) as *const i64))
                        }
                        Type::String => {
                            print!("{:<12}", *(records_ptr.add(offset) as *const i32))
                        }
                    }
                }

                println!();
            }
        }

        println!();
    }
}

fn main() {
    let columns = vec![
        Column {
            name: "id".to_string(),
            kind: Type::Int32,
        },
        Column {
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
