use std::path::Path;

use inkwell::context::Context;
use nom::Parser;

use crate::{parser::module, generator::Generator, passes::remove_scopes::remove_scopes};

mod generator;
mod parser;
mod passes;
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

    test4: I64 = increment(test2)
    "#;

    let parsed = module.parse(text)?;
    println!("{:#?}", parsed);
    let mut parsed = parsed.1;

    remove_scopes(&mut parsed);
    println!("{:#?}", parsed);

    let context = Context::create();
    let mut generator = Generator::new(&context);

    for (name, global) in parsed.globals.iter() {
        if global.arguments.is_empty() {
            if let sir::DataType::Primitive(data_type) = &global.return_type {
                generator.declare_global_primitive_constant(name.clone(), data_type);
            }
        } else {
            generator.declare_global_function(name.clone(), &global.arguments, &global.return_type);
        }
    }

    for (name, global) in parsed.globals.iter() {
        if global.arguments.is_empty() {
            if let sir::DataType::Primitive(_) = global.return_type {
                generator.write_global_primitive_constant(name, &global.body);
            }
        } else {
            generator.write_global_function(name, &global.arguments, &global.body);
        }
    }

    let module = generator.build();
    module.write_bitcode_to_path(&Path::new("scrap.ll"));

    Ok(())
}
