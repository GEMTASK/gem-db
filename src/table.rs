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
    records: Vec<u8>,
    next_records_offset: usize,
    storage: Vec<u8>,
    next_storage_offset: usize,
}

struct Storage {
    records: Vec<u8>,
}

pub enum Query<'a> {
    Eq(&'a str, Value),
    // And(&'a [&'a Query<'a>]),
}

fn field_type_to_column_type(field_type: FieldType) -> ColumnType {
    return match field_type {
        FieldType::Int32 => ColumnType::Int32,
        FieldType::Int64 => ColumnType::Int64,
        FieldType::String => ColumnType::String,
        FieldType::Relation { table } => ColumnType::Relation { table: table },
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
            records: vec![0u8; 32],
            next_records_offset: 0,
            storage: vec![0u8; 16],
            next_storage_offset: 0,
        }
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
                        FieldType::Int32 => {
                            offset += (4 - offset % 4) % 4;
                            column_offsets.push(offset);
                            offset += 4;

                            column_indexes.insert(column.name.clone(), i);
                        }
                        FieldType::Int64 => {
                            offset += (8 - offset % 8) % 8;
                            column_offsets.push(offset);
                            offset += 8;

                            column_indexes.insert(column.name.clone(), i);
                        }
                        FieldType::String => {
                            offset += (4 - offset % 4) % 4;
                            column_offsets.push(offset);
                            offset += 4;

                            column_indexes.insert(column.name.clone(), i);
                        }
                        FieldType::Relation { table } => {
                            offset += (4 - offset % 4) % 4;
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

        self.column_indexes = column_indexes;
        self.column_offsets = column_offsets;
        self.row_width = offset;
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

    pub fn extract_column(&self, record: &[u8], column_index: usize) -> Value {
        let record_ptr = record.as_ptr();
        let storage_ptr: *const u8 = self.storage.as_ptr();

        let offset = self.column_offsets[column_index];

        match &self.columns[column_index].column_type {
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
            ColumnType::Relation { table } => unsafe {
                return Value::Int32(*(record_ptr.add(offset) as *const i32));
            },
        }
    }

    pub fn extract_record(&self, index: usize) -> Vec<Value> {
        let records_ptr: *const u8 = self.records.as_ptr();
        let storage_ptr: *const u8 = self.storage.as_ptr();

        let row_offset = self.row_width as usize * index;

        let mut columns = vec![];

        for (i, column) in self.columns.iter().enumerate() {
            let offset = row_offset + self.column_offsets[i] as usize;

            match &column.column_type {
                ColumnType::Int32 => unsafe {
                    columns.push(Value::Int32(*(records_ptr.add(offset) as *const i32)))
                },
                ColumnType::Int64 => unsafe {
                    columns.push(Value::Int64(*(records_ptr.add(offset) as *const i64)))
                },
                ColumnType::String => unsafe {
                    let string: String;

                    let string_ptr = storage_ptr.add(*(records_ptr.add(offset)) as usize);
                    let length = *(string_ptr as *const u16);

                    let slice = std::slice::from_raw_parts(string_ptr.add(2), length.into());

                    columns.push(Value::String(
                        std::str::from_utf8_unchecked(slice).to_owned(),
                    ));
                },
                ColumnType::Relation { table } => unsafe {
                    columns.push(Value::Int32(*(records_ptr.add(offset) as *const i32)));
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
                        Value::Int32(value) => match *query_value {
                            Value::Int32(query_value) => *value == query_value,
                            _ => false,
                        },
                        Value::Int64(value) => false,
                        Value::String(value) => false,
                        Value::Array(value) => false,
                        Value::Relation(value) => false,
                    };
                }
            }
        }

        return true;
    }

    pub fn select<'a>(&self, query: Option<&'a Query>) -> Vec<Vec<Value>> {
        let mut rows = vec![];

        let records_ptr: *const u8 = self.records.as_ptr();
        let storage_ptr: *const u8 = self.storage.as_ptr();

        for index in 0..self.next_records_offset / self.row_width as usize {
            let mut columns = self.extract_record(index);

            let x = self.filter(query, &columns);

            if !x {
                continue;
            }

            let relation_query = Query::Eq("item_id", columns[0].clone());

            for comment in self.relations.iter() {
                columns.push(Value::Array(
                    (*comment.table.borrow()).select(Some(&relation_query)),
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

        for record in values.iter() {
            for (i, column) in record.iter().enumerate() {
                match &column {
                    Value::Int32(value) => print!("{:<12}", value),
                    Value::Int64(value) => print!("{:<12}", value),
                    Value::String(value) => print!("{:<12}", value),
                    Value::Relation(value) => print!("{:<12}", value),
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
