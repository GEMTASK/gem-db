use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub enum Type {
    Int32,
    Int64,
    String,
    Relation { table: Rc<RefCell<Table>> },
}

#[derive(Clone, Debug)]
pub enum Value {
    Int32(i32),
    Int64(i64),
    String(String),
    Relation(i32),
    Array(Vec<Vec<Value>>),
}

#[derive(Debug)]
pub struct Column {
    pub name: String,
    pub kind: Type,
}

impl Column {
    pub fn new(name: &str, kind: Type) -> Column {
        Self {
            name: name.to_string(),
            kind,
        }
    }
}

#[derive(Debug)]
pub enum RelationType {
    Scalar,
    Array,
}

#[derive(Debug)]
pub struct Relation {
    pub name: String,
    pub key: String,
    pub r#type: RelationType,
    pub table: Rc<RefCell<Table>>,
}

#[derive(Debug)]
pub struct Table {
    name: String,
    pub relations: Vec<Relation>,
    pub columns: Vec<Column>,
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

pub enum Query<'a> {
    Eq(&'a str, Value),
    // And(&'a [&'a Query<'a>]),
}

impl Table {
    pub fn new(name: &str, columns: Vec<Column>) -> Table {
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
                    offset += (4 - offset % 4) % 4;
                    column_offsets[i] = offset;
                    offset += 4;
                }
            }
        }

        Self {
            name: name.to_string(),
            relations: vec![],
            columns,
            column_offsets,
            row_width: offset as u16,
            records: vec![0u8; 32],
            next_records_offset: 0,
            storage: vec![0u8; 16],
            next_storage_offset: 0,
        }
    }

    pub fn add_relations(&mut self, relations: Vec<Relation>) {
        self.relations = relations;
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
                Value::Relation(value) => unsafe {
                    *(records_ptr.add(offset) as *mut i32) = *value;
                },
                Value::Array(values) => {}
            }
        }

        self.next_records_offset += self.row_width as usize;
    }

    pub fn select(&self, query: Option<Query>) -> Vec<Vec<Value>> {
        let mut rows = vec![];
        let mut columns = vec![];

        let records_ptr: *const u8 = self.records.as_ptr();
        let storage_ptr: *const u8 = self.storage.as_ptr();

        match query {
            Some(query) => match query {
                Query::Eq(a, b) => {
                    println!("{:?} == {:?}", a, b);
                }
            },
            None => {}
        }

        for j in 0..self.next_records_offset / self.row_width as usize {
            for (i, field) in self.columns.iter().enumerate() {
                let offset = self.row_width as usize * j + self.column_offsets[i] as usize;

                unsafe {
                    match &field.kind {
                        Type::Int32 => {
                            columns.push(Value::Int32(*(records_ptr.add(offset) as *const i32)))
                        }
                        Type::Int64 => {
                            columns.push(Value::Int64(*(records_ptr.add(offset) as *const i64)))
                        }
                        Type::String => {
                            let string: String;

                            let string_ptr = storage_ptr.add(*(records_ptr.add(offset)) as usize);
                            let length = *(string_ptr as *const u16);

                            let slice =
                                std::slice::from_raw_parts(string_ptr.add(2), length.into());

                            columns.push(Value::String(
                                std::str::from_utf8_unchecked(slice).to_owned(),
                            ));
                        }
                        Type::Relation { table } => {
                            columns.push(Value::Int32(*(records_ptr.add(offset) as *const i32)));
                        }
                    }
                }
            }

            for comment in self.relations.iter() {
                columns.push(Value::Array((*comment.table.borrow()).select(None)));
            }

            rows.push(columns);

            columns = vec![];
        }

        return rows;
    }

    pub fn print(&self, values: Vec<Vec<Value>>) {
        let records_ptr: *const u8 = self.records.as_ptr();
        let storage_ptr: *const u8 = self.storage.as_ptr();

        for (i, field) in self.columns.iter().enumerate() {
            print!("{:<12}", field.name);
        }

        for relation in self.relations.iter() {
            print!("{:<12}", relation.name);
        }

        println!();

        for i in 0..(self.columns.len() + self.relations.len()) {
            print!("{:<12}", "-".repeat(11));
        }

        println!();

        for row in values.iter() {
            for (i, column) in row.iter().enumerate() {
                match &column {
                    Value::Int32(value) => {
                        print!("{:<12}", value);
                    }
                    Value::Int64(value) => {
                        print!("{:<12}", value);
                    }
                    Value::String(value) => {
                        print!("{:<12}", value);
                    }
                    Value::Relation(value) => {
                        print!("{:<12}", value);
                    }
                    Value::Array(values) => {
                        print!("{:?}", values);
                    }
                }
            }

            println!();
        }

        println!();
    }
}
