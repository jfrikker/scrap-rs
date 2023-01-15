use nom::Parser;

use crate::parser::module;

mod parser;
mod sir;

fn main() {
    let text = r#"

    a: I64 = 123i64



    b(abc: I64, def: (I64, I64): I64): I64 = {
        a = 123i64;
        b = a + (a + a);
        b
    }.add(456i64)
    "#;

    println!("{:#?}", module.parse(text));
}
