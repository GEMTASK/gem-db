#[derive(Debug)]
pub enum Type<'a> {
    Int32,
    Int64,
    String,
    Relation { table: &'a Table<'a> },
}

pub enum Value {
    Int32(i32),
    Int64(i64),
    String(String),
    Relation(i32),
}

#[derive(Debug)]
pub struct Column<'a> {
    pub name: String,
    pub kind: Type<'a>,
}

impl<'a> Column<'a> {
    pub fn new(name: &'a str, kind: Type<'a>) -> Column<'a> {
        Self {
            name: name.to_string(),
            kind,
        }
    }
}

#[derive(Debug)]
pub struct Table<'a> {
    name: String,
    pub columns: Vec<Column<'a>>,
    column_offsets: Vec<usize>,
    row_width: u16,
    pub records: Vec<u8>,
    next_records_offset: usize,
    pub storage: Vec<u8>,
    next_storage_offset: usize,
}

struct Storage {
    records: Vec<u8>,
}

impl<'a> Table<'a> {
    pub fn new(name: &'a str, columns: Vec<Column<'a>>) -> Table<'a> {
        let mut column_offsets: Vec<usize> = vec![0; columns.len()];
        let mut offset: usize = 0;

        for (i, field) in columns.iter().enumerate() {
            match &field.kind {
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
                Type::Relation { table } => {
                    //
                }
            }
        }

        Self {
            name: name.to_string(),
            columns,
            column_offsets,
            row_width: offset as u16,
            records: vec![0u8; 32],
            next_records_offset: 0,
            storage: vec![0u8; 16],
            next_storage_offset: 0,
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
                Value::Relation(value) => {
                    //
                }
            }
        }

        self.next_records_offset += self.row_width as usize;
    }

    pub fn select(&self) {
        let records_ptr: *const u8 = self.records.as_ptr();
        let storage_ptr: *const u8 = self.storage.as_ptr();

        for (i, field) in self.columns.iter().enumerate() {
            print!("{:<12}", field.name);
        }

        println!();

        for (i, field) in self.columns.iter().enumerate() {
            print!("{:<12}", "-".repeat(11));
        }

        println!();

        for j in 0..self.next_records_offset / self.row_width as usize {
            for (i, field) in self.columns.iter().enumerate() {
                let offset = self.row_width as usize * j + self.column_offsets[i] as usize;

                unsafe {
                    match &field.kind {
                        Type::Int32 => {
                            print!("{:<12}", *(records_ptr.add(offset) as *const i32));
                        }
                        Type::Int64 => {
                            print!("{:<12}", *(records_ptr.add(offset) as *const i64));
                        }
                        Type::String => {
                            let string: String;

                            let string_ptr = storage_ptr.add(*(records_ptr.add(offset)) as usize);
                            let length = *(string_ptr as *const u16);

                            let slice =
                                std::slice::from_raw_parts(string_ptr.add(2), length.into());

                            print!("{:<12}", std::str::from_utf8_unchecked(slice));
                        }
                        Type::Relation { table } => {
                            //
                        }
                    }
                }
            }

            println!();
        }

        println!();
    }
}
