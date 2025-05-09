pub enum Query<'a> {
    From(&'a str, &'a [&'a Command<'a>]),
}

pub enum Command<'a> {
    Select(&'a [&'a str]),
    Where(&'a Op<'a>),
}

pub enum Op<'a> {
    Eq(&'a str, i32),
    And(&'a [&'a Op<'a>]),
}

#[cfg_attr(rustfmt, rustfmt::skip)]
#[allow(unused_parens)]
const QUERY2: Query = (
    Query::From("items", &[
        &Command::Select(&["id, title"]),
        &Command::Where(
            &Op::And(&[
                &Op::Eq("id", 255)
            ])     
        )
    ])
);

/*

from items
  select id, title
  where id = 1


  items {
    title
    comments {
      comment
    }
  }

*/
