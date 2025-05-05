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
    schema: Schema,
    data: Vec<u8>,
}

fn test() {
    let schema = Schema {
        fields: vec![Field {
            name: "id".to_string(),
            kind: FieldType::Int32,
        }],
    };

    let table = Table {
        name: "id".to_string(),
        schema,
        data: vec![0u8; 16],
    };
}
