use std::collections::HashMap;

use crate::types::{Column, ColumnType, Field, FieldType, Relation, Value};

#[derive(Debug)]
pub struct View {
    fields: Vec<Field>,
    values: Vec<Vec<Value>>,
}

#[derive(Debug)]
pub struct Table {
    pub name: String,
    fields: Vec<Field>,
    relations: Vec<Relation>,
    columns: Vec<Column>,
    column_offsets: Vec<usize>,
    column_indexes: HashMap<String, usize>,
    row_width: usize,
    records: Vec<u32>,
    next_records_offset: usize,
    storage: Vec<u16>,
    next_storage_offset: usize,
}

struct Storage {
    records: Vec<u8>,
}

pub enum Query<'a> {
    Eq(&'a str, &'a Value),
    // And(&'a [&'a Query<'a>]),
}

fn field_type_to_column_type(field_type: FieldType) -> ColumnType {
    return match field_type {
        FieldType::Ulid => ColumnType::Ulid,
        FieldType::Int32 => ColumnType::Int32,
        FieldType::Int64 => ColumnType::Int64,
        FieldType::String => ColumnType::String,
        FieldType::Relation { table } => ColumnType::Int32,
        FieldType::Table {
            key,
            relation_type,
            table,
        } => ColumnType::Int32,
    };
}

impl Table {
    pub fn new(name: &str) -> Table {
        Self {
            name: name.to_string(),
            fields: vec![],
            relations: vec![],
            columns: vec![],
            column_offsets: vec![],
            column_indexes: HashMap::new(),
            row_width: 0,
            records: vec![0u32; 32 / 4],
            next_records_offset: 0,
            storage: vec![0u16; 16 / 2],
            next_storage_offset: 0,
        }
    }

    fn memory_align(offset: usize, alignment: usize) -> usize {
        return if offset % alignment == 0 {
            0
        } else {
            alignment - offset % alignment
        };
    }

    pub fn add_fields(&mut self, fields: Vec<Field>) {
        let mut column_indexes: HashMap<String, usize> = HashMap::new();
        let mut column_offsets: Vec<usize> = vec![];
        let mut offset: usize = 0;

        self.fields = fields.clone();

        for (i, column) in fields.iter().enumerate() {
            match &column.field_type {
                FieldType::Table {
                    key,
                    relation_type,
                    table,
                } => {
                    self.relations.push(Relation {
                        name: column.name.clone(),
                        key: key.clone(),
                        table: table.clone(),
                        relation_type: relation_type.clone(),
                    });
                }
                _ => {
                    match &column.field_type {
                        FieldType::Ulid => {
                            offset += Self::memory_align(offset, 16);
                            column_offsets.push(offset);
                            offset += 16;

                            column_indexes.insert(column.name.clone(), i);
                        }
                        FieldType::Int32 => {
                            offset += Self::memory_align(offset, 4);
                            column_offsets.push(offset);
                            offset += 4;

                            column_indexes.insert(column.name.clone(), i);
                        }
                        FieldType::Int64 => {
                            offset += Self::memory_align(offset, 8);
                            column_offsets.push(offset);
                            offset += 8;

                            column_indexes.insert(column.name.clone(), i);
                        }
                        FieldType::String => {
                            offset += Self::memory_align(offset, 2);
                            column_offsets.push(offset);
                            offset += 2;

                            column_indexes.insert(column.name.clone(), i);
                        }
                        FieldType::Relation { table } => {
                            offset += Self::memory_align(offset, 4);
                            column_offsets.push(offset);
                            offset += 4;

                            column_indexes.insert(column.name.clone(), i);
                        }
                        _ => {}
                    }

                    self.columns.push(Column {
                        name: column.name.clone(),
                        column_type: field_type_to_column_type(column.field_type.clone()),
                    })
                }
            }
        }

        offset += Self::memory_align(offset, 16);

        self.column_indexes = column_indexes;
        self.column_offsets = column_offsets;
        self.row_width = offset;

        println!("{:?} {:?}", self.column_offsets, self.row_width);
    }

    pub fn add_relations(&mut self, relations: Vec<Relation>) {
        self.relations = relations;
    }

    pub fn get_field<T: Copy>(&self, field_index: usize) -> T {
        let ptr: *const u8 = self.records.as_ptr() as *const u8;

        unsafe {
            return *(ptr.add(self.column_offsets[field_index]) as *const T);
        }
    }

    pub fn insert(&mut self, values: &[Value]) {
        let records_ptr: *mut u8 = self.records.as_mut_ptr() as *mut u8;
        let storage_ptr: *mut u8 = self.storage.as_mut_ptr() as *mut u8;

        for (i, field_offset) in self.column_offsets.iter().enumerate() {
            let offset = self.next_records_offset + field_offset;

            match &values[i] {
                Value::Ulid(value) => unsafe {
                    *(records_ptr.add(offset) as *mut u128) = *value;
                },
                Value::Int32(value) => unsafe {
                    *(records_ptr.add(offset) as *mut i32) = *value;
                },
                Value::Int64(value) => unsafe {
                    *(records_ptr.add(offset) as *mut i64) = *value;
                },
                Value::String(value) => unsafe {
                    self.next_storage_offset += Self::memory_align(self.next_storage_offset, 2);

                    *(records_ptr.add(offset) as *mut i32) = self.next_storage_offset as i32;

                    *(storage_ptr.add(self.next_storage_offset) as *mut i16) = value.len() as i16;
                    std::ptr::copy_nonoverlapping(
                        value.as_ptr(),
                        storage_ptr.add(self.next_storage_offset + 4),
                        value.len(),
                    );

                    self.next_storage_offset += value.len() + 2;
                },
                Value::Array(values) => {}
            }
        }

        self.next_records_offset += self.row_width as usize;
    }

    pub fn extract_column(&self, record: &[u8], column_index: usize) -> Value {
        let record_ptr = record.as_ptr();
        let storage_ptr: *const u8 = self.storage.as_ptr() as *const u8;

        let offset = self.column_offsets[column_index];

        match &self.columns[column_index].column_type {
            ColumnType::Ulid => unsafe {
                return Value::Ulid(*(record_ptr.add(offset) as *const u128));
            },
            ColumnType::Int32 => unsafe {
                return Value::Int32(*(record_ptr.add(offset) as *const i32));
            },
            ColumnType::Int64 => unsafe {
                return Value::Int64(*(record_ptr.add(offset) as *const i64));
            },
            ColumnType::String => unsafe {
                let string_ptr = storage_ptr.add(*(record_ptr.add(offset)) as usize);
                let slice =
                    std::slice::from_raw_parts(string_ptr.add(2), *(string_ptr as *const usize));

                return Value::String(std::str::from_utf8_unchecked(slice).to_owned());
            },
        }
    }

    pub fn extract_record(&self, index: usize) -> Vec<Value> {
        let records_ptr: *const u8 = self.records.as_ptr() as *const u8;
        let storage_ptr: *const u8 = self.storage.as_ptr() as *const u8;

        let row_offset = self.row_width as usize * index;

        let mut columns = vec![];

        for (i, column) in self.columns.iter().enumerate() {
            let offset = row_offset + self.column_offsets[i] as usize;

            match &column.column_type {
                ColumnType::Ulid => unsafe {
                    columns.push(Value::Ulid(*(records_ptr.add(offset) as *const u128)))
                },
                ColumnType::Int32 => unsafe {
                    columns.push(Value::Int32(*(records_ptr.add(offset) as *const i32)))
                },
                ColumnType::Int64 => unsafe {
                    columns.push(Value::Int64(*(records_ptr.add(offset) as *const i64)))
                },
                ColumnType::String => unsafe {
                    let string: String;

                    let string_ptr = storage_ptr.add(*(records_ptr.add(offset)) as usize);

                    println!(
                        "> {:?} {:?} {:?} {:?} {:?}",
                        offset,
                        string_ptr,
                        row_offset,
                        *(records_ptr.add(offset)),
                        self.column_offsets[i]
                    );
                    let length = *(string_ptr as *const u16);

                    let slice = std::slice::from_raw_parts(string_ptr.add(2), length.into());

                    columns.push(Value::String(
                        std::str::from_utf8_unchecked(slice).to_owned(),
                    ));
                },
            }
        }

        return columns;
    }

    fn filter<'a>(&self, query_or_none: Option<&'a Query>, values: &'a Vec<Value>) -> bool {
        if let Some(query) = query_or_none {
            match query {
                Query::Eq(column_name, query_value) => {
                    return match &values[self.column_indexes[*column_name]] {
                        Value::Ulid(value) => match *query_value {
                            Value::Ulid(query_value) => *value == *query_value,
                            _ => false,
                        },
                        Value::Int32(value) => match *query_value {
                            Value::Int32(query_value) => *value == *query_value,
                            _ => false,
                        },
                        Value::Int64(value) => false,
                        Value::String(value) => false,
                        Value::Array(value) => false,
                    };
                }
            }
        }

        return true;
    }

    pub fn select<'a>(&self, query: Option<&'a Query>) -> Vec<Vec<Value>> {
        let mut rows = vec![];

        for index in 0..self.next_records_offset / self.row_width as usize {
            let mut columns = self.extract_record(index);

            let x = self.filter(query, &columns);

            if !x {
                continue;
            }

            for relation in self.relations.iter() {
                // println!("{:#?}", relation);

                let relation_query = Query::Eq(&relation.key, &columns[0]);

                columns.push(Value::Array(
                    (*relation.table.borrow()).select(Some(&relation_query)),
                ));
            }

            rows.push(columns);
        }

        // return View {
        //     fields: self.fields,
        //     values: rows,
        // };

        return rows;
    }

    pub fn print(&self, values: Vec<Vec<Value>>) {
        let records_ptr: *const u8 = self.records.as_ptr() as *const u8;
        let storage_ptr: *const u8 = self.storage.as_ptr() as *const u8;

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

        for record in values.iter() {
            for (i, column) in record.iter().enumerate() {
                match &column {
                    Value::Ulid(value) => print!("{:<12}", value),
                    Value::Int32(value) => print!("{:<12}", value),
                    Value::Int64(value) => print!("{:<12}", value),
                    Value::String(value) => print!("{:<12}", value),
                    Value::Array(values) => print!("{:?}", values),
                }
            }

            println!();
        }

        println!();
    }
}

/*

let value = i32::from_le_bytes(self.records[0..4].try_into().unwrap());

*/
