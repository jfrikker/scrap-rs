use std::path::Path;

use inkwell::context::Context;
use nom::Parser;

use crate::{parser::module, generator::Generator};

mod generator;
mod parser;
mod sir;

fn main() -> anyhow::Result<()> {
    let text = r#"
    test1: I64 = 123i64
    test2: I64 = (123i64 + 456i64) + 789i64
    test3: I64 = {
        a = 123i64;
        b = 456i64;
        a + b
    }

    increment(i: I64): I64 = i + 1i64
    "#;

    let parsed = module.parse(text)?;
    println!("{:#?}", parsed);

    let context = Context::create();
    let mut generator = Generator::new(&context);

    for (name, global) in parsed.1.globals.iter() {
        if global.arguments.is_empty() {
            if let sir::DataType::Primitive(data_type) = *global.return_type {
                generator.write_global_primitive_constant(name, data_type, global.body.as_ref());
            }
        } else {
            generator.write_global_function(name, &global.arguments, &global.return_type, &global.body);
        }
    }

    let module = generator.build();
    module.write_bitcode_to_path(&Path::new("scrap.ll"));

    Ok(())
}
