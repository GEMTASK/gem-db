pub enum Value<'a> {
    Int32(i32),
    String(&'a str),
}

pub enum Query<'a> {
    Field(&'a str),
    Array(&'a str, &'a [&'a Command<'a>]),
}

pub enum Command<'a> {
    Select(&'a [&'a Query<'a>]),
    Where(&'a Op<'a>),
}

pub enum Op<'a> {
    Eq(&'a str, Value<'a>),
    And(&'a [&'a Op<'a>]),
}

struct Table {
    //
}

fn apply(query: Query) {
    let mut tables = HashMap::<&str, Table>::new();

    tables.insert("items", Table {});

    match query {
        Field(name) => {
            println!("{}", name);
        }
        Array(name, values) => {
            //
        }
    }
}

use std::collections::HashMap;

use Command::*;
use Op::*;
use Query::*;

#[cfg_attr(rustfmt, rustfmt::skip)]
#[allow(unused_parens)]
const QUERY2: Query = (
    Array("items", &[
        &Select(&[
            &Field("title"),
            &Array("comments", &[
                &Select(&[
                    &Field("comment")
                ])
            ])
        ]),
        &Where(
            &And(&[
                &Eq("id", Value::Int32(255)),
                &Eq("title", Value::String("foo"))
            ])     
        )
    ])
);

/*

items {
  where {
    id == 255
  }
  select {
    title
    comments {
      select {
        comment
      }
    }
  }
}

items
  | where { id == 255 }
  | select { comments }

Declarative vs imperative

*/
