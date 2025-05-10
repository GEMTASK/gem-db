use std::rc::Rc;

#[derive(Debug, Clone)]
enum Value {
    Int32(i32),
    String(Rc<String>),
}

fn test() {
    let a = Value::String(Rc::new("foo".to_string()));
    let b = a.clone();

    println!("{:?}", a);
}
